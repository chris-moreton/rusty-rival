use crate::fen::{algebraic_move_from_move, get_position};
use crate::mvm_test_fens::get_test_fens;
use crate::search::clear_countermoves;
use crate::types::{Move, Score, SearchState, UciState};
use crate::uci::run_command_sync;
use crate::utils::hydrate_move_from_algebraic_move;
use ansi_term::Colour::{Green, Red, Yellow};
use either::{Either, Left, Right};
use num_format::{Locale, ToFormattedString};
use std::thread;
use std::time::Instant;

/// Fixed position set for the deterministic `bench`. Chosen to span opening,
/// middlegame, tactical, and endgame structures so a functional change in any
/// part of search/eval perturbs the node signature. DO NOT reorder or edit these
/// without expecting the signature to change (that is the point of the command).
const BENCH_FENS: [&str; 16] = [
    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
    "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
    "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
    "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
    "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
    "2r3k1/pp2bppp/4pn2/3q4/3P4/2N1PN2/PP3PPP/2RQ1RK1 b - - 0 1",
    "r1bqk2r/1ppp1ppp/p1n2n2/2b1p3/B3P3/2N2N2/PPPP1PPP/R1BQ1RK1 w kq - 0 1",
    "8/7p/p5pb/4k3/P1pPn3/8/P5PP/1rB2RK1 b - d3 0 28",
    "8/7R/1pqp1k2/p3p3/P1n1P3/1Q3P2/2Pr4/1KB5 w - - 2 42",
    "4rrk1/1p1nq3/p7/2p1P1pp/3P2bp/3Q1Bn1/PPPB4/1K2R1NR w - - 0 1",
    "r2q1rk1/4bppp/p2p4/2pP4/3pP3/3Q4/PP1B1PPP/R3R1K1 w - - 0 1",
    "6k1/6p1/6Pp/pppPp2P/1P1Ep3/2K5/8/8 b - - 0 1",
    "3r3k/2r4p/1p1b3q/p4P2/P2Pp3/1B2P3/3BQ1RP/6K1 w - - 0 1",
    "2rr3k/pp3pp1/1nnqbN1p/3pN3/2pP4/2P3Q1/PPB4P/R4RK1 w - - 0 1",
    "8/8/8/8/5kp1/P7/8/1K1N4 w - - 0 1",
    "1k6/5RP1/1P6/1K6/6r1/8/8/8 w - - 0 1",
];

/// Default depth for the deterministic bench. Deep enough that most search
/// features are exercised, shallow enough to finish in a few seconds.
const BENCH_DEFAULT_DEPTH: u8 = 12;

/// Deterministic fixed-depth benchmark (OpenBench style).
///
/// Runs a fixed position set at a fixed depth, single-threaded (`run_command_sync`
/// searches on the calling thread), clearing the TT and history between positions.
/// The total node count is a signature: **any functional change to search or eval
/// changes it, and a pure refactor must not.** Use it as a fast regression check
/// before spending hours on a match.
fn cmd_bench_deterministic(uci_state: &mut UciState, search_state: &mut SearchState, depth: u8) -> Either<String, Option<String>> {
    let show_info = search_state.show_info;
    let saved_fen = uci_state.fen.clone();
    search_state.show_info = false;

    let start = Instant::now();
    let mut total_nodes: u64 = 0;
    let mut total_qnodes: u64 = 0;
    // Accumulated per position: uci.rs resets these on every `go`, so reading
    // them after the loop would report the last position only.
    let (mut tt_probes, mut tt_hits, mut tt_deep, mut tt_taken) = (0u64, 0u64, 0u64, 0u64);
    let (mut scouts, mut rs_lmr, mut rs_full, mut rs_pvs) = (0u64, 0u64, 0u64, 0u64);
    let (mut kids, mut no_cutoff_nodes, mut no_cutoff_children) = (0u64, 0u64, 0u64);
    let (mut cuts, mut cuts_first) = (0u64, 0u64);
    let (mut by_kind, mut by_index) = ([0u64; 7], [0u64; 5]);
    let (mut child_kind, mut child_depth, mut child_node) = ([0u64; 3], [0u64; 5], [0u64; 2]);
    let (mut no_cut_kind, mut cut_kind) = ([0u64; 3], [0u64; 3]);
    let (mut pruned, mut lmr_eligible, mut lmr_applied, mut lmr_researched) = ([0u64; 4], [0u64; 2], [0u64; 2], [0u64; 2]);
    let mut extensions = [0u64; 4];

    for (i, fen) in BENCH_FENS.iter().enumerate() {
        // `ucinewgame` clears the TT and history tables, and `iterative_deepening`
        // clears killers on every `go`. The COUNTERMOVE table is the one thing
        // nothing resets - deliberately, because clearing it costs ~12% of bench
        // in real play (NET-396).
        //
        // That warmth is right for games and wrong for a signature: it carried
        // state from one bench position into the next, so a second bench in the
        // same process returned a different node count (3,310,543 then
        // 3,148,827) despite the comment that used to sit here claiming
        // reproducibility. Caught in review by Codex, who also pointed out that
        // clearing killers here as well was redundant.
        //
        // Cleared here rather than in `ucinewgame`, so the engine keeps its
        // deliberate warmth for real play and only the benchmark is made
        // deterministic.
        run_command_sync(uci_state, search_state, "ucinewgame");
        clear_countermoves(search_state);
        run_command_sync(uci_state, search_state, &format!("position fen {}", fen));
        run_command_sync(uci_state, search_state, &format!("go depth {}", depth));

        let nodes = search_state.nodes;
        total_nodes += nodes;
        total_qnodes += search_state.qnodes;
        tt_probes += search_state.tt_probes;
        tt_hits += search_state.tt_hits;
        tt_deep += search_state.tt_deep_enough;
        tt_taken += search_state.tt_slot_taken;
        scouts += search_state.scout_searches;
        rs_lmr += search_state.research_lmr_full;
        rs_full += search_state.research_full_depth;
        rs_pvs += search_state.research_pvs;
        kids += search_state.children_searched;
        no_cutoff_nodes += search_state.no_cutoff_nodes;
        no_cutoff_children += search_state.no_cutoff_children;
        cuts += search_state.cutoffs;
        cuts_first += search_state.cutoffs_first_move;
        for (acc, n) in by_kind.iter_mut().zip(search_state.cutoff_by_kind.iter()) {
            *acc += n;
        }
        for (acc, n) in by_index.iter_mut().zip(search_state.cutoff_by_index.iter()) {
            *acc += n;
        }
        for (acc, n) in child_kind.iter_mut().zip(search_state.children_by_kind) {
            *acc += n;
        }
        for (acc, n) in child_depth.iter_mut().zip(search_state.children_by_depth) {
            *acc += n;
        }
        for (acc, n) in child_node.iter_mut().zip(search_state.children_by_node_type) {
            *acc += n;
        }
        for (acc, n) in no_cut_kind.iter_mut().zip(search_state.no_cutoff_children_by_kind) {
            *acc += n;
        }
        for (acc, n) in cut_kind.iter_mut().zip(search_state.cutoff_node_children_by_kind) {
            *acc += n;
        }
        for (acc, n) in pruned.iter_mut().zip(search_state.pruned_by_reason) {
            *acc += n;
        }
        for (acc, n) in lmr_eligible.iter_mut().zip(search_state.lmr_eligible_by_kind) {
            *acc += n;
        }
        for (acc, n) in lmr_applied.iter_mut().zip(search_state.lmr_applied_by_kind) {
            *acc += n;
        }
        for (acc, n) in lmr_researched.iter_mut().zip(search_state.lmr_researched_by_kind) {
            *acc += n;
        }
        for (acc, n) in extensions.iter_mut().zip(search_state.extension_children) {
            *acc += n;
        }
        println!(
            "Position {:>2}/{}: {:>12} nodes",
            i + 1,
            BENCH_FENS.len(),
            nodes.to_formatted_string(&Locale::en)
        );
    }

    let elapsed = start.elapsed();
    let millis = elapsed.as_millis().max(1) as u64;
    let nps = total_nodes * 1000 / millis;

    println!("===========================");
    println!("Depth         : {}", depth);
    println!("Time          : {} ms", millis.to_formatted_string(&Locale::en));
    println!("Nodes searched: {}", total_nodes);
    println!("NPS           : {}", nps.to_formatted_string(&Locale::en));

    // Move-ordering quality (NET-493). Of every beta cutoff, the share that came
    // from the first move tried. A node that cut on move 6 searched five subtrees
    // it did not need, so this ratio drives the effective branching factor and
    // therefore the depth reached in a fixed time.
    if cuts > 0 {
        println!(
            "Cutoffs       : {} ({:.1}% on first move)",
            cuts.to_formatted_string(&Locale::en),
            cuts_first as f64 / cuts as f64 * 100.0
        );
    }

    // Which heuristic supplied the cutting move, and how far down the list it
    // sat. Together these say *why* the first-move rate is short of the low-90s
    // strong engines reach - a weak TT share points at replacement policy, a
    // long tail past move 3 points at quiet-move ranking.
    if cuts > 0 {
        let tot = cuts as f64;
        let kinds = [
            "TT move",
            "capture",
            "promo",
            "killer",
            "distant killer",
            "countermove",
            "other quiet",
        ];
        print!("  by heuristic :");
        for (name, n) in kinds.iter().zip(by_kind.iter()) {
            print!(" {} {:.1}%", name, *n as f64 / tot * 100.0);
        }
        println!();
        let idx = ["1st", "2nd", "3rd", "4-6th", "7th+"];
        print!("  by position  :");
        for (name, n) in idx.iter().zip(by_index.iter()) {
            print!(" {} {:.1}%", name, *n as f64 / tot * 100.0);
        }
        println!();
    }

    // Transposition table effectiveness. A low hit rate means re-searching
    // subtrees already solved - a constant factor invisible to both the
    // branching factor and the ordering stats.
    if tt_probes > 0 {
        let p = tt_probes as f64;
        println!(
            "TT probes     : {} · hit {:.1}% · deep enough {:.1}% · slot taken by other pos {:.1}%",
            tt_probes.to_formatted_string(&Locale::en),
            tt_hits as f64 / p * 100.0,
            tt_deep as f64 / p * 100.0,
            tt_taken as f64 / p * 100.0
        );
    }

    // Re-search cost. Every re-search repeats a subtree already searched once,
    // and none of it appears in the branching factor or the cutoff histogram.
    if scouts > 0 {
        let sc = scouts as f64;
        println!(
            "Scout searches: {} · LMR re-search (reduced, full window) {:.1}% · then full depth {:.1}% · PVS re-search {:.1}%",
            scouts.to_formatted_string(&Locale::en),
            rs_lmr as f64 / sc * 100.0,
            rs_full as f64 / sc * 100.0,
            rs_pvs as f64 / sc * 100.0
        );
    }

    // Width at nodes that completed the move loop with no beta cutoff - the
    // last place a constant factor could hide. Cut nodes
    // are already known to be well ordered; this is what happens where nothing
    // cuts and every move must be searched unless pruned or reduced.
    if no_cutoff_nodes > 0 {
        println!(
            "Children      : {} searched · no-cutoff nodes {} · {:.2} children each",
            kids.to_formatted_string(&Locale::en),
            no_cutoff_nodes.to_formatted_string(&Locale::en),
            no_cutoff_children as f64 / no_cutoff_nodes as f64
        );
    }

    // NET-1155: the detailed build is deliberately separate from production;
    // these counters have measurable low-single-digit cost when enabled. The three child-kind and both
    // node-type buckets are exclusive and must each reconcile exactly to
    // children_searched.
    #[cfg(feature = "search-width-diagnostics")]
    {
        assert_eq!(child_kind.iter().sum::<u64>(), kids, "child-kind counters must reconcile");
        assert_eq!(child_depth.iter().sum::<u64>(), kids, "child-depth counters must reconcile");
        assert_eq!(child_node.iter().sum::<u64>(), kids, "node-window counters must reconcile");
        assert_eq!(
            no_cut_kind.iter().sum::<u64>(),
            no_cutoff_children,
            "no-cut child kinds must reconcile"
        );
        println!(
            "  child kinds  : quiet {} · capture {} · promotion {} · total {}",
            child_kind[0].to_formatted_string(&Locale::en),
            child_kind[1].to_formatted_string(&Locale::en),
            child_kind[2].to_formatted_string(&Locale::en),
            child_kind.iter().sum::<u64>().to_formatted_string(&Locale::en),
        );
        println!(
            "  node windows : full {} · scout {} · total {}",
            child_node[0].to_formatted_string(&Locale::en),
            child_node[1].to_formatted_string(&Locale::en),
            child_node.iter().sum::<u64>().to_formatted_string(&Locale::en),
        );
        println!(
            "  depth buckets: 0-2 {} · 3-5 {} · 6-9 {} · 10-15 {} · 16+ {}",
            child_depth[0].to_formatted_string(&Locale::en),
            child_depth[1].to_formatted_string(&Locale::en),
            child_depth[2].to_formatted_string(&Locale::en),
            child_depth[3].to_formatted_string(&Locale::en),
            child_depth[4].to_formatted_string(&Locale::en),
        );
        println!(
            "  completed    : no-cut quiet/capture/promo {}/{}/{} · cutoff-node {}/{}/{}",
            no_cut_kind[0], no_cut_kind[1], no_cut_kind[2], cut_kind[0], cut_kind[1], cut_kind[2],
        );
        println!(
            "  pruned moves : SEE {} · alpha/futility {} · LMP {} · rejected before make {}",
            pruned[0].to_formatted_string(&Locale::en),
            pruned[1].to_formatted_string(&Locale::en),
            pruned[2].to_formatted_string(&Locale::en),
            pruned[3].to_formatted_string(&Locale::en),
        );
        println!(
            "  LMR quiet    : eligible {} · applied {} · re-searched {}",
            lmr_eligible[0].to_formatted_string(&Locale::en),
            lmr_applied[0].to_formatted_string(&Locale::en),
            lmr_researched[0].to_formatted_string(&Locale::en),
        );
        println!(
            "  LMR captures : eligible {} · applied {} · re-searched {}",
            lmr_eligible[1].to_formatted_string(&Locale::en),
            lmr_applied[1].to_formatted_string(&Locale::en),
            lmr_researched[1].to_formatted_string(&Locale::en),
        );
        println!(
            "  extensions   : check {} · pawn7 {} · passed {} · singular {}",
            extensions[0].to_formatted_string(&Locale::en),
            extensions[1].to_formatted_string(&Locale::en),
            extensions[2].to_formatted_string(&Locale::en),
            extensions[3].to_formatted_string(&Locale::en),
        );
    }

    // Where the tree actually is. A growth-rate problem shows up in the
    // branching factor; a constant-factor problem shows up as an outsized
    // quiescence share.
    if total_qnodes > 0 {
        let main = total_nodes.saturating_sub(total_qnodes);
        println!(
            "  main search : {:>12} ({:.1}%)",
            main.to_formatted_string(&Locale::en),
            main as f64 / total_nodes as f64 * 100.0
        );
        println!(
            "  quiescence  : {:>12} ({:.1}%)",
            total_qnodes.to_formatted_string(&Locale::en),
            total_qnodes as f64 / total_nodes as f64 * 100.0
        );
    }

    search_state.show_info = show_info;
    uci_state.fen = saved_fen;
    Right(None)
}

pub fn cmd_benchmark(uci_state: &mut UciState, search_state: &mut SearchState, parts: Vec<&str>) -> Either<String, Option<String>> {
    // `bench`            -> deterministic node-count signature at the default depth
    // `bench depth <N>`  -> deterministic signature at depth N
    // `bench <millis>`   -> legacy wall-clock tactical suite
    if parts.len() == 1 {
        return cmd_bench_deterministic(uci_state, search_state, BENCH_DEFAULT_DEPTH);
    }
    if parts.len() == 3 && parts[1] == "depth" {
        return match parts[2].parse::<u8>() {
            Ok(d) if d >= 1 => cmd_bench_deterministic(uci_state, search_state, d),
            _ => Left::<String, Option<String>>("usage: bench depth <1-250>".parse().unwrap()),
        };
    }
    if parts.len() != 2 {
        return Left::<String, Option<String>>("usage: bench | bench depth <N> | bench <millis>".parse().unwrap());
    }

    let start = Instant::now();
    let positions = get_test_fens();
    let total = positions.len();

    let show_info = search_state.show_info;
    search_state.show_info = false;
    // `bench cat` must be a usage error, not a panic that kills the engine
    // (NET-369)
    let millis: u32 = match parts.get(1).unwrap().parse() {
        Ok(m) => m,
        Err(_) => {
            search_state.show_info = show_info;
            return Left::<String, Option<String>>("usage: bench | bench depth <N> | bench <millis>".parse().unwrap());
        }
    };

    let mut total_nodes = 0;
    let mut total_correct = 0;
    let mut total_tested = 0;
    let mut total_expected = 0;

    for p in positions {
        let fen = p.0;
        let expected_move = p.1;
        let expected_millis = p.3;

        println!("-------------------------------------------------------------------------------------");
        println!("{} Expect {} in {}ms", fen, expected_move, expected_millis);
        println!("-------------------------------------------------------------------------------------");
        let min_diff = p.2;
        let mut owned = "position fen ".to_owned();
        owned.push_str(fen);

        let mut these_millis = 250;

        loop {
            run_command_sync(uci_state, search_state, "ucinewgame");
            run_command_sync(uci_state, search_state, &owned);

            let mut main_uci_state = uci_state.clone();
            let mut main_search_state = search_state.clone();
            let main_handle = thread::Builder::new()
                .stack_size(16 * 1024 * 1024)
                .spawn(move || get_main_move(&mut main_uci_state, &mut main_search_state, &these_millis))
                .expect("Failed to spawn main search thread");

            let mut second_uci_state = uci_state.clone();
            let mut second_search_state = search_state.clone();
            let second_handle = thread::Builder::new()
                .stack_size(16 * 1024 * 1024)
                .spawn(move || {
                    let position = get_position(fen);
                    let raw_move = hydrate_move_from_algebraic_move(&position, expected_move.to_string());
                    get_secondary_move(&mut second_uci_state, &mut second_search_state, raw_move, &these_millis)
                })
                .expect("Failed to spawn secondary search thread");

            let (best_move, best_score, main_search_nodes) = main_handle.join().unwrap();
            let (second_best_move, second_best_score) = second_handle.join().unwrap();

            let alg_move = algebraic_move_from_move(best_move);

            total_nodes += main_search_nodes;
            let mut tick;

            let score_diff = best_score - second_best_score;
            let score_is_good = score_diff >= min_diff;

            if alg_move == expected_move && score_is_good {
                total_tested += 1;
                total_correct += 1;
                if these_millis <= expected_millis {
                    total_expected += 1;
                }
                tick = "\u{2705}";
                show_result(
                    &mut total_correct,
                    &mut total_tested,
                    fen,
                    expected_move,
                    best_score,
                    main_search_nodes,
                    second_best_move,
                    second_best_score,
                    alg_move,
                    &mut tick,
                    score_diff,
                    score_is_good,
                    these_millis,
                    expected_millis,
                    true,
                );
                break;
            } else {
                these_millis *= 2;

                if these_millis > millis {
                    total_tested += 1;
                    tick = "\u{274C}";
                    show_result(
                        &mut total_correct,
                        &mut total_tested,
                        fen,
                        expected_move,
                        best_score,
                        main_search_nodes,
                        second_best_move,
                        second_best_score,
                        alg_move,
                        &mut tick,
                        score_diff,
                        score_is_good,
                        these_millis / 2,
                        expected_millis,
                        true,
                    );
                    break;
                }

                tick = " ";
                show_result(
                    &mut total_correct,
                    &mut total_tested,
                    fen,
                    expected_move,
                    best_score,
                    main_search_nodes,
                    second_best_move,
                    second_best_score,
                    alg_move,
                    &mut tick,
                    score_diff,
                    score_is_good,
                    these_millis / 2,
                    expected_millis,
                    false,
                );
            }
        }
    }
    let duration = start.elapsed();
    println!("Time elapsed is: {:?}", duration);
    println!(
        "Correct: {}/{}",
        Yellow.paint(total_correct.to_string()),
        Yellow.paint(total.to_string())
    );
    println!("Within Expected Time: {:?}/{}", total_expected, total);
    let nps = (total_nodes as f64 / start.elapsed().as_millis() as f64) * 1000.0;

    println!(
        "{} nodes {} nps",
        total_nodes.to_formatted_string(&Locale::en),
        &*(nps as u64).to_string()
    );

    search_state.show_info = show_info;

    Right(None)
}

#[allow(clippy::too_many_arguments)]
fn show_result(
    total_correct: &mut i32,
    total_tested: &mut i32,
    _fen: &str,
    expected_move: &str,
    best_score: Score,
    main_search_nodes: u64,
    second_best_move: Move,
    second_best_score: Score,
    alg_move: String,
    tick: &mut &str,
    score_diff: Score,
    score_is_good: bool,
    millis_taken: u32,
    expected_millis: u32,
    show_fen: bool,
) {
    if show_fen {
        println!(
            "{} Nodes {} {}/{}",
            tick,
            main_search_nodes.to_formatted_string(&Locale::en),
            Yellow.paint(total_correct.to_string()),
            Yellow.paint(total_tested.to_string()),
        );
    }

    println!(
        " \u{27A5} [1st {} Score {}] [2nd {} Score {}] [Diff {}] [Within {}ms]",
        if alg_move == expected_move {
            Green.paint(&alg_move)
        } else {
            Red.paint(&alg_move)
        },
        best_score,
        algebraic_move_from_move(second_best_move),
        second_best_score,
        if score_is_good {
            Green.paint(score_diff.to_string())
        } else {
            Red.paint(score_diff.to_string())
        },
        if expected_millis >= millis_taken {
            Green.paint(millis_taken.to_string())
        } else {
            Red.paint(millis_taken.to_string())
        }
    );
}

fn get_main_move(uci_state: &mut UciState, search_state: &mut SearchState, millis: &u32) -> (Move, Score, u64) {
    run_command_sync(uci_state, search_state, &format!("go movetime {}", millis));
    let best_move = search_state.current_best.0[0];
    let best_score = search_state.current_best.1;
    (best_move, best_score, search_state.nodes)
}

fn get_secondary_move(uci_state: &mut UciState, search_state: &mut SearchState, best_move: Move, millis: &u32) -> (Move, Score) {
    search_state.ignore_root_move = best_move;

    run_command_sync(uci_state, search_state, &format!("go movetime {}", millis));
    let second_best_move = search_state.current_best.0[0];
    let second_best_score = search_state.current_best.1;
    (second_best_move, second_best_score)
}

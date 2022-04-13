use crate::fen::{algebraic_move_from_move, get_position};
use crate::mvm_test_fens::get_test_fens;
use crate::types::{Move, Score, SearchState, UciState};
use crate::uci::run_command;
use crate::utils::hydrate_move_from_algebraic_move;
use ansi_term::Colour::{Green, Red, White};
use either::{Either, Left, Right};
use num_format::{Locale, ToFormattedString};
use std::thread;
use std::time::Instant;

pub fn cmd_benchmark(uci_state: &mut UciState, search_state: &mut SearchState, parts: Vec<&str>) -> Either<String, Option<String>> {
    if parts.len() != 2 {
        return Left::<String, Option<String>>("usage: bench <millis>".parse().unwrap());
    }

    let start = Instant::now();
    let positions = get_test_fens();
    let total = positions.len();

    let show_info = search_state.show_info;
    search_state.show_info = false;
    let millis: u32 = parts.get(1).unwrap().to_string().parse().unwrap();

    let mut total_nodes = 0;
    let mut total_correct = 0;
    let mut total_tested = 0;
    run_command(uci_state, search_state, "setoption name MultiPV value 2");

    for p in positions {
        let fen = p.0;
        let expected_move = p.1;
        let min_diff = p.2;
        run_command(uci_state, search_state, "ucinewgame");
        let mut owned = "position fen ".to_owned();
        owned.push_str(fen);

        run_command(uci_state, search_state, &owned);

        let mut main_uci_state = uci_state.clone();
        let mut main_search_state = search_state.clone();
        let main_handle = thread::spawn(move || get_main_move(&mut main_uci_state, &mut main_search_state, &millis));

        let mut second_uci_state = uci_state.clone();
        let mut second_search_state = search_state.clone();
        let second_handle = thread::spawn(move || {
            let position = get_position(fen);
            let raw_move = hydrate_move_from_algebraic_move(&position, expected_move.to_string());
            get_secondary_move(&mut second_uci_state, &mut second_search_state, raw_move, &millis)
        });

        let (best_move, best_score, main_search_nodes) = main_handle.join().unwrap();
        let (second_best_move, second_best_score) = second_handle.join().unwrap();

        let alg_move = algebraic_move_from_move(best_move);

        total_tested += 1;
        total_nodes += main_search_nodes;
        let mut tick = "\u{274C}";

        let score_diff = best_score - second_best_score;
        let score_is_good = score_diff >= min_diff;

        if alg_move == expected_move && score_is_good {
            total_correct += 1;
            tick = "\u{2705}";
        }

        println!(
            "\n{} {}: Nodes {} Expected {} {}/{}",
            tick,
            fen,
            main_search_nodes.to_formatted_string(&Locale::en),
            expected_move,
            total_correct,
            total_tested
        );

        println!(
            " \u{27A5} [Best {} Score {}] [Second best {} Score {}] [Diff {}]",
            if alg_move == expected_move {
                Green.paint(&alg_move)
            } else {
                Red.paint(&alg_move)
            },
            best_score,
            if alg_move == expected_move {
                algebraic_move_from_move(second_best_move)
            } else {
                "N/A".to_string()
            },
            if alg_move == expected_move {
                second_best_score.to_string()
            } else {
                "N/A".to_string()
            },
            if alg_move == expected_move {
                if score_is_good {
                    Green.paint(score_diff.to_string())
                } else {
                    Red.paint(score_diff.to_string())
                }
            } else {
                White.paint("N/A".to_string())
            },
        );
    }
    let duration = start.elapsed();
    println!("Time elapsed is: {:?}", duration);
    println!("Correct: {:?}/{}", total_correct, total);
    let nps = (total_nodes as f64 / start.elapsed().as_millis() as f64) * 1000.0;

    println!(
        "{} nodes {} nps",
        total_nodes.to_formatted_string(&Locale::en),
        &*(nps as u64).to_string()
    );

    search_state.show_info = show_info;

    Right(None)
}

fn get_main_move(uci_state: &mut UciState, search_state: &mut SearchState, millis: &u32) -> (Move, Score, u64) {
    run_command(uci_state, search_state, &format!("go movetime {}", millis));
    let best_move = search_state.current_best.0[0];
    let best_score = search_state.current_best.1;
    (best_move, best_score, search_state.nodes)
}

fn get_secondary_move(uci_state: &mut UciState, search_state: &mut SearchState, best_move: Move, millis: &u32) -> (Move, Score) {
    search_state.ignore_root_move = best_move;

    run_command(uci_state, search_state, &format!("go movetime {}", millis));
    let second_best_move = search_state.current_best.0[0];
    let second_best_score = search_state.current_best.1;
    (second_best_move, second_best_score)
}

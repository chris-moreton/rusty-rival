use crate::fen::{algebraic_move_from_move, get_position};
use crate::mvm_test_fens::get_test_fens;
use crate::types::{Move, Score, SearchState, UciState};
use crate::uci::run_command;
use crate::utils::hydrate_move_from_algebraic_move;
use ansi_term::Colour::{Green, Red, White, Yellow};
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
            run_command(uci_state, search_state, "ucinewgame");
            run_command(uci_state, search_state, &owned);

            let mut main_uci_state = uci_state.clone();
            let mut main_search_state = search_state.clone();
            let main_handle = thread::spawn(move || get_main_move(&mut main_uci_state, &mut main_search_state, &these_millis));

            let mut second_uci_state = uci_state.clone();
            let mut second_search_state = search_state.clone();
            let second_handle = thread::spawn(move || {
                let position = get_position(fen);
                let raw_move = hydrate_move_from_algebraic_move(&position, expected_move.to_string());
                get_secondary_move(&mut second_uci_state, &mut second_search_state, raw_move, &these_millis)
            });

            let (best_move, best_score, main_search_nodes) = main_handle.join().unwrap();
            let (second_best_move, second_best_score) = second_handle.join().unwrap();

            let alg_move = algebraic_move_from_move(best_move);

            total_nodes += main_search_nodes;
            let mut tick = "\u{274C}";

            let score_diff = best_score - second_best_score;
            let score_is_good = score_diff >= min_diff;

            if alg_move == expected_move && score_is_good {
                total_tested += 1;
                total_correct += 1;
                if these_millis <= expected_millis {
                    total_expected += 1;
                }
                tick = "\u{2705}";
                show_result(&mut total_correct, &mut total_tested, fen, expected_move, best_score, main_search_nodes, second_best_move, second_best_score, alg_move, &mut tick, score_diff, score_is_good, these_millis, expected_millis, true);
                break
            } else {
                these_millis *= 2;

                if these_millis > millis {
                    total_tested += 1;
                    tick = "\u{274C}";
                    show_result(&mut total_correct, &mut total_tested, fen, expected_move, best_score, main_search_nodes, second_best_move, second_best_score, alg_move, &mut tick, score_diff, score_is_good, these_millis / 2, expected_millis, true);
                    break
                }

                tick = " ";
                show_result(&mut total_correct, &mut total_tested, fen, expected_move, best_score, main_search_nodes, second_best_move, second_best_score, alg_move, &mut tick, score_diff, score_is_good, these_millis / 2, expected_millis, false);
            }
        }
    }
    let duration = start.elapsed();
    println!("Time elapsed is: {:?}", duration);
    println!("Correct: {:?}/{}", Yellow.paint(total_correct.to_string()), Yellow.paint(total.to_string()));
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
fn show_result(total_correct: &mut i32, total_tested: &mut i32, fen: &str, expected_move: &str, best_score: Score, main_search_nodes: u64, second_best_move: Move, second_best_score: Score, alg_move: String, tick: &mut &str, score_diff: Score, score_is_good: bool, millis_taken: u32, expected_millis: u32, show_fen: bool) {

    if show_fen {
        println!(
            "{} Nodes {} {}/{}",
            tick,
            main_search_nodes.to_formatted_string(&Locale::en),
            total_correct,
            total_tested
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
        second_best_score.to_string(),
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

//! Self-play training-data generation (NET-319).
//!
//! Plays the engine against itself at a fixed node count and writes quiet,
//! labelled positions for NNUE training. The output is the plain-text format
//! bullet ingests directly:
//!
//! ```text
//! <fen> | <score> | <wdl>
//! ```
//!
//! where `score` is centipawns **from white's point of view** and `wdl` is the
//! eventual game result, also from white's point of view (`1.0` white win,
//! `0.5` draw, `0.0` black win).
//!
//! Text rather than a packed binary format is deliberate: bullet's binary layout
//! is a moving target and is not verifiable from this repo (the trainer lives
//! elsewhere), whereas this format is self-describing, diffable, and converted by
//! bullet's own tooling. The conversion step is thin and cheap to correct; the
//! expensive part — actually playing the games — is format-agnostic.
//!
//! ## Filtering
//!
//! A position is only recorded when it is a useful training target:
//!
//! * not in check (the eval of a forced position teaches little),
//! * the chosen move is not a capture (avoids labelling mid-exchange positions
//!   with a score that only makes sense after the recapture),
//! * the score is not a mate score (mate distances are not an eval signal),
//! * it is past the random opening plies.
//!
//! These are the standard filters; without them a large fraction of the data is
//! actively misleading.

use crate::fen::{algebraic_move_from_move, get_fen, get_position};
use crate::make_move::make_move;
use crate::moves::{generate_moves, is_check};
use crate::search::MATE_START;
use crate::types::{default_search_state, default_uci_state, Move, Position, Score, SearchState, UciState, WHITE};
use crate::uci::run_command_sync;
use crate::utils::is_capture;
use either::{Either, Left, Right};
use num_format::{Locale, ToFormattedString};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::Instant;

/// Default number of random plies played from the start position before the
/// engine takes over. Enough to decorrelate games without reaching positions
/// that are already lost.
const DEFAULT_RANDOM_PLIES: usize = 8;

/// Hard cap on game length, so a shuffling drawn game cannot run forever.
const MAX_GAME_PLIES: usize = 400;

/// Adjudicate a game as decided once one side is this far ahead for
/// `ADJUDICATION_PLIES` consecutive plies. Saves a lot of time playing out
/// completely won positions.
const ADJUDICATION_SCORE: Score = 1000;
const ADJUDICATION_PLIES: usize = 6;

/// Reject a random opening whose evaluation is already outside this margin.
///
/// Uniformly random plies regularly produce wrecked positions (a king walking to
/// the fourth rank by move five), which generate decisive games full of positions
/// no real game would ever reach. Without this filter an early sample run
/// produced **20 games with zero draws** — a clear sign the openings, not the
/// play, were deciding the results.
const OPENING_MAX_IMBALANCE: Score = 400;

/// Game outcome from white's point of view.
#[derive(Clone, Copy, PartialEq)]
enum Outcome {
    WhiteWin,
    Draw,
    BlackWin,
}

impl Outcome {
    fn wdl(self) -> &'static str {
        match self {
            Outcome::WhiteWin => "1.0",
            Outcome::Draw => "0.5",
            Outcome::BlackWin => "0.0",
        }
    }
}

/// A position recorded during play, labelled with the result once the game ends.
struct Sample {
    fen: String,
    /// Centipawns from white's point of view.
    score: Score,
}

/// All legal moves for a position (generate_moves is pseudo-legal).
fn legal_moves(position: &Position) -> Vec<Move> {
    let mover = position.mover;
    generate_moves(position)
        .into_iter()
        .filter(|m| {
            let mut next = *position;
            make_move(position, *m, &mut next);
            !is_check(&next, mover)
        })
        .collect()
}

/// Terminal-position check. Returns `None` if the game continues.
///
/// Repetition and the fifty-move rule are handled by the caller, which has the
/// position history; this only covers conditions visible from the position alone.
fn terminal_outcome(position: &Position, moves: &[Move]) -> Option<Outcome> {
    if !moves.is_empty() {
        return None;
    }
    if is_check(position, position.mover) {
        // Side to move is mated.
        Some(if position.mover == WHITE {
            Outcome::BlackWin
        } else {
            Outcome::WhiteWin
        })
    } else {
        Some(Outcome::Draw)
    }
}

/// Play one self-play game, appending its labelled positions to `out`.
///
/// Returns the outcome, or `None` if the random opening produced an immediately
/// finished game (in which case the caller should just start another).
fn play_game(
    uci_state: &mut UciState,
    search_state: &mut SearchState,
    rng: &mut StdRng,
    nodes: u64,
    random_plies: usize,
    out: &mut Vec<Sample>,
) -> Option<Outcome> {
    const START_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

    // Clear the TT and history between games. Without this, entries from the
    // previous game bias the early search of the next one and the generated data
    // stops being independent per game.
    run_command_sync(uci_state, search_state, "ucinewgame");

    let mut position = get_position(START_FEN);
    let mut played: Vec<String> = Vec::new();

    // --- Random opening -----------------------------------------------------
    for _ in 0..random_plies {
        let moves = legal_moves(&position);
        if moves.is_empty() {
            return None;
        }
        let m = moves[rng.gen_range(0..moves.len())];
        played.push(algebraic_move_from_move(m));
        let mut next = position;
        make_move(&position, m, &mut next);
        position = next;
    }
    if legal_moves(&position).is_empty() {
        return None;
    }

    // Reject openings that are already lopsided (see OPENING_MAX_IMBALANCE).
    let opening_cmd = format!("position fen {} moves {}", START_FEN, played.join(" "));
    run_command_sync(uci_state, search_state, &opening_cmd);
    run_command_sync(uci_state, search_state, &format!("go nodes {}", nodes));
    let opening_stm_score = search_state.current_best.1;
    if opening_stm_score.abs() > OPENING_MAX_IMBALANCE {
        return None;
    }

    // --- Engine self-play ---------------------------------------------------
    let start_index = out.len();
    let mut decisive_streak = 0usize;
    let mut streak_favours_white = false;
    let mut outcome = Outcome::Draw;

    for _ in 0..MAX_GAME_PLIES {
        let moves = legal_moves(&position);
        if let Some(o) = terminal_outcome(&position, &moves) {
            outcome = o;
            break;
        }
        // Fifty-move rule. Repetition is covered by the engine's own draw
        // detection during search; at game level the move counter is the
        // cheap, unambiguous stopping condition.
        if position.half_moves >= 100 {
            outcome = Outcome::Draw;
            break;
        }

        // Search this position. Rebuilding the `position ... moves` command each
        // ply keeps search_state.history correct, which is what makes the
        // engine's repetition detection work.
        let cmd = if played.is_empty() {
            format!("position fen {}", START_FEN)
        } else {
            format!("position fen {} moves {}", START_FEN, played.join(" "))
        };
        run_command_sync(uci_state, search_state, &cmd);
        run_command_sync(uci_state, search_state, &format!("go nodes {}", nodes));

        let best_move = search_state.current_best.0[0];
        let stm_score = search_state.current_best.1;
        if best_move == 0 {
            break;
        }

        // Score from white's point of view.
        let white_score = if position.mover == WHITE { stm_score } else { -stm_score };

        // --- Filtering ------------------------------------------------------
        let is_mate_score = stm_score.abs() >= MATE_START;
        let quiet = !is_check(&position, position.mover) && !is_capture(&position, best_move) && !is_mate_score;
        if quiet {
            out.push(Sample {
                fen: get_fen(&position),
                score: white_score,
            });
        }

        // --- Adjudication ---------------------------------------------------
        let favours_white = white_score > 0;
        if white_score.abs() >= ADJUDICATION_SCORE {
            // A decisive ply that flips the favoured side STARTS a new streak
            // rather than clearing it: the flip ply is itself decisive, and
            // resetting to 0 discarded that (NET-374). Restructured so the
            // trigger check runs on every decisive ply, including the flip -
            // the old shape would silently break if ADJUDICATION_PLIES were 1.
            decisive_streak = if decisive_streak > 0 && favours_white == streak_favours_white {
                decisive_streak + 1
            } else {
                1
            };
            streak_favours_white = favours_white;
            if decisive_streak >= ADJUDICATION_PLIES {
                outcome = if favours_white { Outcome::WhiteWin } else { Outcome::BlackWin };
                break;
            }
        } else {
            decisive_streak = 0;
        }

        if is_mate_score {
            // A forced mate is a decided game; no need to play it out.
            outcome = if white_score > 0 { Outcome::WhiteWin } else { Outcome::BlackWin };
            break;
        }

        played.push(algebraic_move_from_move(best_move));
        let mut next = position;
        make_move(&position, best_move, &mut next);
        position = next;
    }

    // Discard games that produced nothing useful.
    if out.len() == start_index {
        return None;
    }
    Some(outcome)
}

/// Base RNG seed. Each worker offsets this by its index so runs are
/// reproducible for a given thread count but workers never duplicate games.
const BASE_SEED: u64 = 0x0052_17A1_1EAD_u64;

/// Per-worker transposition table size. Datagen searches are tiny (a few
/// thousand nodes), so a large table wastes memory that matters once multiplied
/// by the worker count — 8 workers at the 128MB default would reserve 1GB of
/// almost entirely untouched table.
const WORKER_HASH_MB: usize = 16;

/// Stack size for worker threads. Search recurses deeply and the default 2MB
/// thread stack overflows; this matches the engine's other spawned search threads.
const WORKER_STACK_BYTES: usize = 16 * 1024 * 1024;

/// `datagen <games> <nodes> <output> [random-plies] [threads]`
///
/// Games are independent, so generation is embarrassingly parallel: each worker
/// owns its own `UciState`, `SearchState` (including its own TT) and RNG, and
/// streams finished games back to this thread, which is the sole writer. That
/// keeps the output append-ordered and free of interleaving with no locking on
/// the hot path.
pub fn cmd_datagen(_uci_state: &mut UciState, _search_state: &mut SearchState, parts: Vec<&str>) -> Either<String, Option<String>> {
    const USAGE: &str = "usage: datagen <games> <nodes-per-move> <output-file> [random-plies] [threads]";

    if parts.len() < 4 || parts.len() > 6 {
        return Left(USAGE.to_string());
    }
    let games: usize = match parts[1].parse() {
        Ok(g) if g > 0 => g,
        _ => return Left(USAGE.to_string()),
    };
    let nodes: u64 = match parts[2].parse() {
        Ok(n) if n > 0 => n,
        _ => return Left(USAGE.to_string()),
    };
    let path = parts[3].to_string();
    let random_plies: usize = if parts.len() >= 5 {
        match parts[4].parse() {
            Ok(p) => p,
            _ => return Left(USAGE.to_string()),
        }
    } else {
        DEFAULT_RANDOM_PLIES
    };
    let threads: usize = if parts.len() == 6 {
        match parts[5].parse() {
            Ok(t) if t > 0 => t,
            _ => return Left(USAGE.to_string()),
        }
    } else {
        thread::available_parallelism().map(|n| n.get()).unwrap_or(1)
    };

    let file = match File::create(&path) {
        Ok(f) => f,
        Err(e) => return Left(format!("datagen: cannot write {}: {}", path, e)),
    };
    let mut writer = BufWriter::new(file);

    let start = Instant::now();
    let stop = Arc::new(AtomicBool::new(false));
    let (tx, rx) = mpsc::channel::<(Vec<Sample>, Outcome)>();

    let mut handles = Vec::with_capacity(threads);
    for worker in 0..threads {
        let tx = tx.clone();
        let stop = Arc::clone(&stop);
        let handle = thread::Builder::new().stack_size(WORKER_STACK_BYTES).spawn(move || {
            let mut uci_state = default_uci_state();
            let mut search_state = default_search_state();
            search_state.show_info = false;
            run_command_sync(
                &mut uci_state,
                &mut search_state,
                &format!("setoption name Hash value {}", WORKER_HASH_MB),
            );

            let mut rng = StdRng::seed_from_u64(BASE_SEED.wrapping_add(worker as u64));

            while !stop.load(Ordering::Relaxed) {
                let mut samples: Vec<Sample> = Vec::new();
                if let Some(outcome) = play_game(&mut uci_state, &mut search_state, &mut rng, nodes, random_plies, &mut samples) {
                    // A send error means the collector has finished and dropped
                    // the receiver, so there is nothing left to do.
                    if tx.send((samples, outcome)).is_err() {
                        break;
                    }
                }
            }
        });
        match handle {
            Ok(h) => handles.push(h),
            Err(e) => return Left(format!("datagen: cannot spawn worker {}: {}", worker, e)),
        }
    }
    // Drop this thread's sender, or the channel never closes.
    drop(tx);

    let mut total_positions: u64 = 0;
    let mut played_games: usize = 0;
    let mut results = [0usize; 3]; // white win, draw, black win
    let mut write_error: Option<String> = None;

    for (samples, outcome) in rx {
        for s in &samples {
            if writeln!(writer, "{} | {} | {}", s.fen, s.score, outcome.wdl()).is_err() {
                write_error = Some(format!("datagen: write failed to {}", path));
                break;
            }
        }
        if write_error.is_some() {
            break;
        }

        total_positions += samples.len() as u64;
        played_games += 1;
        results[match outcome {
            Outcome::WhiteWin => 0,
            Outcome::Draw => 1,
            Outcome::BlackWin => 2,
        }] += 1;

        if played_games.is_multiple_of(10) {
            println!(
                "{} games, {} positions ({:.0} pos/game), {:.1}s",
                played_games,
                total_positions.to_formatted_string(&Locale::en),
                total_positions as f64 / played_games as f64,
                start.elapsed().as_secs_f64()
            );
        }

        if played_games >= games {
            break;
        }
    }

    // Signal the workers and wait. They check `stop` between games, so a run
    // ends once the games already in flight finish rather than being cut off
    // mid-game and discarding that work.
    stop.store(true, Ordering::Relaxed);
    for h in handles {
        let _ = h.join();
    }

    let _ = writer.flush();

    if let Some(e) = write_error {
        return Left(e);
    }

    let secs = start.elapsed().as_secs_f64();
    println!("===========================");
    println!("Games         : {}", played_games);
    println!("Positions     : {}", total_positions.to_formatted_string(&Locale::en));
    println!("W/D/L         : {} / {} / {}", results[0], results[1], results[2]);
    println!("Threads       : {}", threads);
    println!("Time          : {:.1}s", secs);
    println!("Positions/sec : {:.0}", total_positions as f64 / secs.max(0.001));
    println!("Output        : {}", path);

    Right(None)
}

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
use crate::search::{clear_countermoves, MATE_START};
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
    // `ucinewgame` deliberately leaves the countermove table warm (see
    // `clear_countermoves`). For datagen that warmth is the one thing standing
    // between us and reproducible output, so drop it here.
    clear_countermoves(search_state);

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

/// Base RNG seed. Mixed with the GAME INDEX rather than the worker index, so a
/// given game number always produces the same game regardless of how many
/// threads are running. That is what makes a run both reproducible and
/// resumable: workers claim indices from a shared counter, so which worker
/// plays which game no longer changes the data.
const BASE_SEED: u64 = 0x0052_17A1_1EAD_u64;

/// How many random openings to try for one game index before giving up. The
/// imbalance filter rejects roughly one opening in ten, so exhausting this is
/// vanishingly unlikely; the bound exists only so a pathological setting cannot
/// spin forever.
const MAX_OPENING_ATTEMPTS: usize = 64;

/// Games per output shard. Shards are the unit of resumability: one is written
/// to a temporary path and atomically renamed, so a shard on disk is proof its
/// games are complete and flushed. Small enough that a crash loses little,
/// large enough that zstd has a useful window.
const GAMES_PER_SHARD: usize = 500;

/// Derive a game's seed from its index. SplitMix64 finalizer, so adjacent game
/// numbers give unrelated streams.
fn game_seed(index: usize) -> u64 {
    let mut z = BASE_SEED.wrapping_add((index as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15));
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Path of a sealed shard - one holding exactly `GAMES_PER_SHARD` games.
fn shard_path(base: &str, n: usize) -> String {
    format!("{}.{:05}.zst", base, n)
}

/// Path of the trailing short shard, which holds `games` games rather than a
/// full `GAMES_PER_SHARD`.
///
/// The count is in the filename because it is the only record of how many games
/// a shard represents: a shard stores positions, and the number of positions per
/// game varies. Without it, resuming a dataset that did not end on a shard
/// boundary would credit the short shard with a full `GAMES_PER_SHARD` and
/// silently generate fewer games than asked for.
fn short_shard_path(base: &str, n: usize, games: usize) -> String {
    format!("{}.{:05}.p{}.zst", base, n, games)
}

/// Games already on disk for this output base, and the shard index to write next.
///
/// Sealed shards are counted from zero and stop at the first gap, so a run that
/// died mid-write resumes from a contiguous prefix rather than trusting stray
/// files. A short shard is only ever written at the end of a run, so at most one
/// can exist and it always sits at the first free index.
fn scan_shards(base: &str) -> (usize, usize) {
    let mut sealed = 0;
    while std::path::Path::new(&shard_path(base, sealed)).exists() {
        sealed += 1;
    }
    match short_shard_games(base, sealed) {
        Some(games) => (sealed * GAMES_PER_SHARD + games, sealed + 1),
        None => (sealed * GAMES_PER_SHARD, sealed),
    }
}

/// Game count of the short shard at `index`, if one is present.
fn short_shard_games(base: &str, index: usize) -> Option<usize> {
    let path = std::path::Path::new(base);
    let dir = match path.parent() {
        Some(p) if !p.as_os_str().is_empty() => p.to_path_buf(),
        _ => std::path::PathBuf::from("."),
    };
    let stem = path.file_name()?.to_str()?;
    let prefix = format!("{}.{:05}.p", stem, index);

    for entry in std::fs::read_dir(dir).ok()?.flatten() {
        let name = entry.file_name();
        let name = name.to_str()?;
        if let Some(rest) = name.strip_prefix(&prefix) {
            if let Some(digits) = rest.strip_suffix(".zst") {
                if let Ok(games) = digits.parse::<usize>() {
                    return Some(games);
                }
            }
        }
    }
    None
}

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
    const USAGE: &str = "usage: datagen <games> <nodes-per-move> <output-base> [random-plies] [threads]";

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

    // Resume from whatever is already on disk. A shard exists only after its
    // atomic rename, so it is proof its games are complete and durable; an
    // interrupted run loses only the games that were in flight.
    //
    // Resuming is only sound because a game index reproduces the same game
    // regardless of thread count or scheduling - see `game_seed` and the
    // per-game reset in `play_game`.
    let (resumed_games, next_shard) = scan_shards(&path);
    let mut shard_index = next_shard;
    if resumed_games >= games {
        return Right(Some(format!(
            "datagen: {} already holds {} games - nothing to do",
            path, resumed_games
        )));
    }
    if resumed_games > 0 {
        println!(
            "Resuming: {} games already on disk in {} shard(s), {} to go",
            resumed_games,
            shard_index,
            games - resumed_games
        );
    }

    let start = Instant::now();
    let stop = Arc::new(AtomicBool::new(false));
    let (tx, rx) = mpsc::channel::<(Vec<Sample>, Outcome)>();
    // Workers claim game indices from here, so no two play the same game and
    // the assignment does not depend on the thread count.
    let next_game = Arc::new(std::sync::atomic::AtomicUsize::new(resumed_games));

    let mut handles = Vec::with_capacity(threads);
    for worker in 0..threads {
        let tx = tx.clone();
        let stop = Arc::clone(&stop);
        let next_game = Arc::clone(&next_game);
        let handle = thread::Builder::new().stack_size(WORKER_STACK_BYTES).spawn(move || {
            let mut uci_state = default_uci_state();
            let mut search_state = default_search_state();
            search_state.show_info = false;
            run_command_sync(
                &mut uci_state,
                &mut search_state,
                &format!("setoption name Hash value {}", WORKER_HASH_MB),
            );

            while !stop.load(Ordering::Relaxed) {
                // Claim the next game index; its seed is a pure function of that
                // index, so the game played is identical on any thread count and
                // on a resumed run.
                let index = next_game.fetch_add(1, Ordering::Relaxed);
                if index >= games {
                    break;
                }

                // One claimed index yields exactly ONE game. play_game returns
                // None when the random opening is rejected (already lopsided, or
                // no legal moves), which is common enough that letting a
                // rejection consume the index would make "games completed" and
                // "indices consumed" drift apart by the rejection rate - and the
                // resume arithmetic depends on those being the same number.
                // Retrying under a derived seed keeps the mapping exact and
                // still deterministic.
                let mut samples: Vec<Sample> = Vec::new();
                let mut produced = None;
                for attempt in 0..MAX_OPENING_ATTEMPTS {
                    if stop.load(Ordering::Relaxed) {
                        break;
                    }
                    samples.clear();
                    let mut rng = StdRng::seed_from_u64(game_seed(index).wrapping_add(attempt as u64));
                    if let Some(outcome) = play_game(&mut uci_state, &mut search_state, &mut rng, nodes, random_plies, &mut samples) {
                        produced = Some(outcome);
                        break;
                    }
                }
                if let Some(outcome) = produced {
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

    // Positions buffered for the shard currently being filled.
    let mut shard_buf: Vec<u8> = Vec::new();
    let mut games_in_shard = 0usize;

    for (samples, outcome) in rx {
        for s in &samples {
            if writeln!(shard_buf, "{} | {} | {}", s.fen, s.score, outcome.wdl()).is_err() {
                write_error = Some("datagen: formatting failed".to_string());
                break;
            }
        }
        if write_error.is_some() {
            break;
        }
        games_in_shard += 1;
        if games_in_shard == GAMES_PER_SHARD {
            if let Err(e) = flush_shard(&shard_path(&path, shard_index), &shard_buf) {
                write_error = Some(e);
                break;
            }
            shard_index += 1;
            shard_buf.clear();
            games_in_shard = 0;
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

    // Trailing short shard - the run finished (or was stopped) mid-shard. Its
    // game count goes in the filename so a later run can resume from an exact
    // total rather than assuming a full shard's worth (see `short_shard_path`).
    if write_error.is_none() && games_in_shard > 0 {
        if let Err(e) = flush_shard(&short_shard_path(&path, shard_index, games_in_shard), &shard_buf) {
            write_error = Some(e);
        } else {
            shard_index += 1;
        }
    }

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
    println!("Shards        : {} ({})", shard_index, shard_path(&path, 0));

    Right(None)
}

/// Write one shard: zstd-compress into a temporary file, fsync, then rename.
///
/// The rename is the commit point. A shard that exists is therefore complete
/// and durable, which is exactly the property `existing_shards` relies on to
/// resume - a half-written file can never be mistaken for finished work.
fn flush_shard(final_path: &str, data: &[u8]) -> Result<(), String> {
    let tmp_path = format!("{}.tmp", final_path);

    let file = File::create(&tmp_path).map_err(|e| format!("datagen: cannot create {}: {}", tmp_path, e))?;
    let mut encoder = zstd::Encoder::new(BufWriter::new(file), ZSTD_LEVEL).map_err(|e| format!("datagen: zstd init failed: {}", e))?;
    encoder
        .write_all(data)
        .map_err(|e| format!("datagen: write failed to {}: {}", tmp_path, e))?;
    let mut inner = encoder.finish().map_err(|e| format!("datagen: zstd finish failed: {}", e))?;
    inner.flush().map_err(|e| format!("datagen: flush failed: {}", e))?;
    inner
        .get_ref()
        .sync_all()
        .map_err(|e| format!("datagen: fsync failed on {}: {}", tmp_path, e))?;
    drop(inner);

    std::fs::rename(&tmp_path, final_path).map_err(|e| format!("datagen: cannot rename {} -> {}: {}", tmp_path, final_path, e))?;
    Ok(())
}

/// zstd level. 3 is the default and compresses FEN text ~6x at a speed far
/// above what datagen produces, so the writer never becomes the bottleneck.
const ZSTD_LEVEL: i32 = 3;

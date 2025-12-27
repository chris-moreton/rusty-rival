#!/usr/bin/env python3
"""
Engine vs Engine competition harness using python-chess.

Usage:
    # Single match between two engines (uses opening book by default)
    ./scripts/compete.py v1-baseline v2-weak-queen --games 100 --time 1.0

    # League (round-robin) between multiple engines
    ./scripts/compete.py v1-baseline v2-weak-queen v3-test --league --games 10 --time 0.5

    # Disable opening book (start from initial position)
    ./scripts/compete.py v1-baseline v2-weak-queen --games 10 --no-book
"""

import argparse
import math
import random
import sys
import chess
import chess.engine
import chess.pgn
from datetime import datetime
from pathlib import Path
from itertools import combinations


# Opening positions (FEN) - balanced positions after 4-8 moves from various openings
# Each position will be played twice (once with each engine as white)
# 50 positions = 100 unique games
OPENING_BOOK = [
    # Sicilian Defense variations
    ("rnbqkb1r/pp2pppp/3p1n2/8/3NP3/2N5/PPP2PPP/R1BQKB1R b KQkq - 1 5", "Sicilian Open"),
    ("r1bqkb1r/pp1ppppp/2n2n2/8/3NP3/8/PPP2PPP/RNBQKB1R w KQkq - 4 5", "Sicilian Nc6"),
    ("rnbqkb1r/pp2pppp/3p1n2/8/3NP3/8/PPP1BPPP/RNBQK2R b KQkq - 1 5", "Sicilian Najdorf"),
    ("r1bqkb1r/pp2pppp/2np1n2/8/3NP3/2N5/PPP2PPP/R1BQKB1R w KQkq - 0 6", "Sicilian Scheveningen"),
    ("r1bqkb1r/pp3ppp/2nppn2/8/3NP3/2N1B3/PPP2PPP/R2QKB1R w KQkq - 0 7", "Sicilian Dragon"),

    # Italian Game / Giuoco Piano
    ("r1bqk1nr/pppp1ppp/2n5/2b1p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 4 4", "Italian Game"),
    ("r1bqkb1r/pppp1ppp/2n2n2/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 4 4", "Two Knights"),
    ("r1bqk2r/pppp1ppp/2n2n2/2b1p3/2B1P3/3P1N2/PPP2PPP/RNBQK2R b KQkq - 0 5", "Italian Giuoco Piano"),

    # Ruy Lopez
    ("r1bqkbnr/pppp1ppp/2n5/1B2p3/4P3/5N2/PPPP1PPP/RNBQK2R b KQkq - 3 3", "Ruy Lopez"),
    ("r1bqkb1r/1ppp1ppp/p1n2n2/1B2p3/4P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 0 5", "Ruy Lopez Morphy"),
    ("r1bqkb1r/1ppp1ppp/p1n2n2/4p3/B3P3/5N2/PPPP1PPP/RNBQK2R b KQkq - 1 5", "Ruy Lopez Closed"),
    ("r1bqkb1r/2pp1ppp/p1n2n2/1p2p3/4P3/1B3N2/PPPP1PPP/RNBQK2R w KQkq - 0 6", "Ruy Lopez Marshall"),

    # Queen's Gambit
    ("rnbqkb1r/ppp2ppp/4pn2/3p4/2PP4/2N5/PP2PPPP/R1BQKBNR w KQkq - 2 4", "QGD"),
    ("rnbqkb1r/ppp1pppp/5n2/3p4/2PP4/8/PP2PPPP/RNBQKBNR w KQkq - 2 3", "QG Accepted"),
    ("rnbqkb1r/ppp2ppp/4pn2/3p4/2PP4/5N2/PP2PPPP/RNBQKB1R w KQkq - 2 4", "QGD Orthodox"),
    ("rnbqkb1r/p1p2ppp/1p2pn2/3p4/2PP4/5NP1/PP2PP1P/RNBQKB1R w KQkq - 0 5", "QGD Tartakower"),

    # King's Indian Defense
    ("rnbqk2r/ppp1ppbp/3p1np1/8/2PPP3/2N5/PP3PPP/R1BQKBNR w KQkq - 0 5", "King's Indian"),
    ("rnbq1rk1/ppp1ppbp/3p1np1/8/2PPP3/2N2N2/PP2BPPP/R1BQK2R b KQ - 3 6", "KID Classical"),
    ("rnbq1rk1/ppp1ppbp/3p1np1/8/2PPP3/2N2N2/PP3PPP/R1BQKB1R w KQ - 2 6", "KID Samisch"),
    ("rnbq1rk1/ppp2pbp/3p1np1/4p3/2PPP3/2N2N2/PP2BPPP/R1BQK2R w KQ - 0 7", "KID Mar del Plata"),

    # French Defense
    ("rnbqkbnr/ppp2ppp/4p3/3pP3/3P4/8/PPP2PPP/RNBQKBNR b KQkq - 0 3", "French Advance"),
    ("rnbqkb1r/ppp2ppp/4pn2/3p4/3PP3/2N5/PPP2PPP/R1BQKBNR w KQkq - 2 4", "French Nc3"),
    ("rnbqkb1r/ppp2ppp/4pn2/3pP3/3P4/2N5/PPP2PPP/R1BQKBNR b KQkq - 1 4", "French Steinitz"),
    ("rnbqk2r/ppp1bppp/4pn2/3p4/3PP3/2N2N2/PPP2PPP/R1BQKB1R w KQkq - 4 5", "French Classical"),

    # Caro-Kann
    ("rnbqkbnr/pp2pppp/2p5/3pP3/3P4/8/PPP2PPP/RNBQKBNR b KQkq - 0 3", "Caro-Kann Advance"),
    ("rnbqkb1r/pp2pppp/2p2n2/3p4/3PP3/2N5/PPP2PPP/R1BQKBNR w KQkq - 2 4", "Caro-Kann Classical"),
    ("rnbqkb1r/pp2pppp/5n2/2pp4/3PP3/2N5/PPP2PPP/R1BQKBNR w KQkq - 0 4", "Caro-Kann Panov"),

    # English Opening
    ("rnbqkbnr/pppp1ppp/8/4p3/2P5/8/PP1PPPPP/RNBQKBNR w KQkq e6 0 2", "English vs e5"),
    ("rnbqkb1r/pppppppp/5n2/8/2P5/5N2/PP1PPPPP/RNBQKB1R b KQkq - 2 2", "English Nf3"),
    ("rnbqkbnr/pp1ppppp/8/2p5/2P5/5N2/PP1PPPPP/RNBQKB1R b KQkq - 1 2", "English Symmetrical"),

    # Slav Defense
    ("rnbqkb1r/pp2pppp/2p2n2/3p4/2PP4/5N2/PP2PPPP/RNBQKB1R w KQkq - 2 4", "Slav Defense"),
    ("rnbqkb1r/p3pppp/2p2n2/1p1p4/2PP4/5N2/PP2PPPP/RNBQKB1R w KQkq - 0 5", "Slav Chebanenko"),

    # Nimzo-Indian
    ("rnbqk2r/pppp1ppp/4pn2/8/1bPP4/2N5/PP2PPPP/R1BQKBNR w KQkq - 2 4", "Nimzo-Indian"),
    ("rnbqk2r/pppp1ppp/4pn2/8/1bPP4/2N2N2/PP2PPPP/R1BQKB1R b KQkq - 3 4", "Nimzo-Indian Classical"),
    ("rnbqk2r/pppp1ppp/4pn2/8/1bPP4/2N1P3/PP3PPP/R1BQKBNR b KQkq - 0 4", "Nimzo-Indian Rubinstein"),

    # Scotch Game
    ("r1bqkbnr/pppp1ppp/2n5/4p3/3PP3/5N2/PPP2PPP/RNBQKB1R b KQkq d3 0 3", "Scotch Game"),
    ("r1bqkb1r/pppp1ppp/2n2n2/4N3/3PP3/8/PPP2PPP/RNBQKB1R b KQkq - 0 4", "Scotch Four Knights"),

    # Petrov Defense
    ("rnbqkb1r/pppp1ppp/5n2/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 2 3", "Petrov Defense"),
    ("rnbqkb1r/pppp1ppp/8/4p3/4n3/5N2/PPPPQPPP/RNB1KB1R b KQkq - 3 4", "Petrov Classical"),

    # Pirc Defense
    ("rnbqkb1r/ppp1pppp/3p1n2/8/3PP3/8/PPP2PPP/RNBQKBNR w KQkq - 2 3", "Pirc Defense"),
    ("rnbqkb1r/ppp1pp1p/3p1np1/8/3PP3/2N5/PPP2PPP/R1BQKBNR w KQkq - 0 4", "Pirc Austrian"),

    # Scandinavian
    ("rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 2", "Scandinavian"),
    ("rnb1kbnr/ppp1pppp/8/3q4/8/2N5/PPPP1PPP/R1BQKBNR b KQkq - 1 3", "Scandinavian Qxd5"),

    # London System
    ("rnbqkb1r/ppp1pppp/3p1n2/8/3P1B2/5N2/PPP1PPPP/RN1QKB1R b KQkq - 3 3", "London System"),
    ("rnbqkb1r/ppp1pppp/5n2/3p4/3P1B2/5N2/PPP1PPPP/RN1QKB1R b KQkq - 3 3", "London vs d5"),

    # Catalan
    ("rnbqkb1r/pppp1ppp/4pn2/8/2PP4/6P1/PP2PP1P/RNBQKBNR b KQkq - 0 3", "Catalan"),
    ("rnbqkb1r/ppp2ppp/4pn2/3p4/2PP4/6P1/PP2PP1P/RNBQKBNR w KQkq - 0 4", "Catalan Open"),

    # Grunfeld
    ("rnbqkb1r/ppp1pp1p/5np1/3p4/2PP4/2N5/PP2PPPP/R1BQKBNR w KQkq d6 0 4", "Grunfeld"),
    ("rnbqkb1r/ppp1pp1p/6p1/3n4/3P4/2N5/PP2PPPP/R1BQKBNR w KQkq - 0 5", "Grunfeld Exchange"),

    # Vienna Game
    ("rnbqkb1r/pppp1ppp/5n2/4p3/4P3/2N5/PPPP1PPP/R1BQKBNR w KQkq - 2 3", "Vienna Game"),

    # King's Gambit
    ("rnbqkbnr/pppp1ppp/8/4p3/4PP2/8/PPPP2PP/RNBQKBNR b KQkq f3 0 2", "King's Gambit"),
]


def calculate_elo_difference(wins: int, losses: int, draws: int) -> tuple[float, float]:
    """
    Calculate Elo difference and error margin from match results.
    Returns (elo_diff, error_margin) where positive means engine1 is stronger.
    """
    total = wins + losses + draws
    if total == 0:
        return 0.0, 0.0

    # Score from engine1's perspective
    score = (wins + draws * 0.5) / total

    # Avoid division by zero at extremes
    if score <= 0.001:
        return -800.0, 100.0
    if score >= 0.999:
        return 800.0, 100.0

    # Elo difference formula
    elo_diff = -400 * math.log10(1 / score - 1)

    # Error margin (approximate 95% confidence interval)
    # Based on standard error of proportion
    std_error = math.sqrt(score * (1 - score) / total)

    # Convert to Elo error (derivative of Elo formula)
    if 0.01 < score < 0.99:
        elo_error = 400 * std_error / (score * (1 - score) * math.log(10))
        elo_error = min(elo_error, 200)  # Cap at reasonable value
    else:
        elo_error = 100

    return elo_diff, elo_error


def get_engine_path(name: str, engine_dir: Path) -> Path:
    path = engine_dir / name / "rusty-rival"
    if not path.exists():
        print(f"Error: Engine not found: {path}")
        sys.exit(1)
    return path


def play_game(engine1_path: Path, engine2_path: Path,
              engine1_name: str, engine2_name: str,
              time_per_move: float,
              start_fen: str = None,
              opening_name: str = None) -> tuple[str, chess.pgn.Game]:
    """Play a single game and return (result, pgn_game)."""
    if start_fen:
        board = chess.Board(start_fen)
    else:
        board = chess.Board()

    engine1 = chess.engine.SimpleEngine.popen_uci(str(engine1_path))
    engine2 = chess.engine.SimpleEngine.popen_uci(str(engine2_path))

    engines = [engine1, engine2]

    game = chess.pgn.Game()
    game.headers["Event"] = "Engine Match"
    game.headers["Date"] = datetime.now().strftime("%Y.%m.%d")
    game.headers["White"] = engine1_name
    game.headers["Black"] = engine2_name
    if start_fen:
        game.headers["FEN"] = start_fen
        game.headers["SetUp"] = "1"
    if opening_name:
        game.headers["Opening"] = opening_name

    # Set up the game from the FEN position
    if start_fen:
        game.setup(board)

    node = game

    try:
        while not board.is_game_over():
            engine = engines[0] if board.turn == chess.WHITE else engines[1]
            result = engine.play(board, chess.engine.Limit(time=time_per_move))
            board.push(result.move)
            node = node.add_variation(result.move)

    except Exception as e:
        print(f"  Error during game: {e}")
    finally:
        engine1.quit()
        engine2.quit()

    game.headers["Result"] = board.result()
    return board.result(), game


def run_match(engine1_name: str, engine2_name: str, engine_dir: Path,
              num_games: int, time_per_move: float, pgn_file: Path,
              use_opening_book: bool = True) -> dict:
    """Run a match between two engines, alternating colors."""

    engine1_path = get_engine_path(engine1_name, engine_dir)
    engine2_path = get_engine_path(engine2_name, engine_dir)

    results = {"1-0": 0, "0-1": 0, "1/2-1/2": 0, "*": 0}
    engine1_points = 0.0
    engine2_points = 0.0

    # Prepare openings - each opening played twice (once per side)
    if use_opening_book:
        # Calculate how many openings we need
        num_opening_pairs = (num_games + 1) // 2

        # Shuffle and cycle through openings
        openings = OPENING_BOOK.copy()
        random.shuffle(openings)

        # Extend if we need more openings than available
        while len(openings) < num_opening_pairs:
            extra = OPENING_BOOK.copy()
            random.shuffle(extra)
            openings.extend(extra)

        openings = openings[:num_opening_pairs]
    else:
        openings = None

    print(f"\n{'='*60}")
    print(f"Match: {engine1_name} vs {engine2_name}")
    print(f"Games: {num_games}, Time: {time_per_move}s/move")
    if use_opening_book:
        print(f"Opening book: {len(OPENING_BOOK)} positions (randomized)")
    else:
        print("Opening book: disabled")
    print(f"{'='*60}\n", flush=True)

    games = []

    for i in range(num_games):
        # Get opening for this game pair
        if openings:
            opening_idx = i // 2
            opening_fen, opening_name = openings[opening_idx]
        else:
            opening_fen, opening_name = None, None

        # Alternate colors
        if i % 2 == 0:
            white, black = engine1_name, engine2_name
            white_path, black_path = engine1_path, engine2_path
            is_engine1_white = True
        else:
            white, black = engine2_name, engine1_name
            white_path, black_path = engine2_path, engine1_path
            is_engine1_white = False

        result, game = play_game(white_path, black_path, white, black,
                                  time_per_move, opening_fen, opening_name)
        games.append(game)
        results[result] += 1

        # Calculate points
        if result == "1-0":
            if is_engine1_white:
                engine1_points += 1
            else:
                engine2_points += 1
        elif result == "0-1":
            if is_engine1_white:
                engine2_points += 1
            else:
                engine1_points += 1
        elif result == "1/2-1/2":
            engine1_points += 0.5
            engine2_points += 0.5

        # Live update
        game_num = i + 1
        opening_info = f" [{opening_name}]" if opening_name else ""
        print(f"Game {game_num:3d}/{num_games}: {white} vs {black} -> {result}{opening_info}  "
              f"| Score: {engine1_name} {engine1_points:.1f} - {engine2_points:.1f} {engine2_name}",
              flush=True)

    # Save PGN
    with open(pgn_file, "w") as f:
        for game in games:
            print(game, file=f)
            print(file=f)

    # Calculate Elo difference
    # From engine1's perspective: wins are when engine1 won
    engine1_wins = int(engine1_points - (results["1/2-1/2"] * 0.5))
    engine1_losses = int(engine2_points - (results["1/2-1/2"] * 0.5))
    elo_diff, elo_error = calculate_elo_difference(
        engine1_wins, engine1_losses, results["1/2-1/2"]
    )

    # Print summary
    print(f"\n{'='*60}")
    print("FINAL RESULTS")
    print(f"{'='*60}")
    print(f"{engine1_name}: {engine1_points:.1f} points")
    print(f"{engine2_name}: {engine2_points:.1f} points")
    print(f"\nResults: +{engine1_wins} -{engine1_losses} ={results['1/2-1/2']}")
    win_rate = engine1_points / num_games * 100
    print(f"Win rate: {win_rate:.1f}%")

    if elo_diff > 0:
        print(f"\nElo difference: {engine1_name} is +{elo_diff:.0f} (±{elo_error:.0f}) stronger")
    else:
        print(f"\nElo difference: {engine2_name} is +{-elo_diff:.0f} (±{elo_error:.0f}) stronger")

    print(f"\nPGN saved to: {pgn_file}")
    print(f"{'='*60}\n")

    return {
        engine1_name: engine1_points,
        engine2_name: engine2_points,
        "results": results,
        "elo_diff": elo_diff,
        "elo_error": elo_error
    }


def run_league(engine_names: list[str], engine_dir: Path,
               games_per_pairing: int, time_per_move: float, results_dir: Path,
               use_opening_book: bool = True):
    """Run a round-robin league between multiple engines."""

    print(f"\n{'='*60}")
    print("LEAGUE MODE - Round Robin Tournament")
    print(f"Engines: {', '.join(engine_names)}")
    print(f"Games per pairing: {games_per_pairing}")
    print(f"Time: {time_per_move}s/move")
    print(f"{'='*60}\n")

    standings = {name: 0.0 for name in engine_names}
    pairings = list(combinations(engine_names, 2))

    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")

    for engine1, engine2 in pairings:
        pgn_file = results_dir / f"league_{engine1}_vs_{engine2}_{timestamp}.pgn"
        match_results = run_match(engine1, engine2, engine_dir,
                                  games_per_pairing, time_per_move, pgn_file,
                                  use_opening_book)
        standings[engine1] += match_results[engine1]
        standings[engine2] += match_results[engine2]

    # Print final standings
    print(f"\n{'='*60}")
    print("FINAL LEAGUE STANDINGS")
    print(f"{'='*60}")

    sorted_standings = sorted(standings.items(), key=lambda x: -x[1])
    for i, (name, points) in enumerate(sorted_standings, 1):
        print(f"{i}. {name}: {points:.1f} points")

    print(f"{'='*60}\n")


def main():
    parser = argparse.ArgumentParser(description="Chess engine competition harness")
    parser.add_argument("engines", nargs="+", help="Engine version names (e.g., v1-baseline)")
    parser.add_argument("--games", "-g", type=int, default=100,
                        help="Number of games (default: 100)")
    parser.add_argument("--time", "-t", type=float, default=1.0,
                        help="Time per move in seconds (default: 1.0)")
    parser.add_argument("--league", "-l", action="store_true",
                        help="Run round-robin league (requires 3+ engines)")
    parser.add_argument("--no-book", action="store_true",
                        help="Disable opening book (start all games from initial position)")

    args = parser.parse_args()

    script_dir = Path(__file__).parent
    engine_dir = script_dir.parent / "engines"
    results_dir = script_dir.parent / "results" / "competitions"
    results_dir.mkdir(parents=True, exist_ok=True)

    use_opening_book = not args.no_book

    if args.league:
        if len(args.engines) < 3:
            print("Error: League mode requires at least 3 engines")
            sys.exit(1)
        run_league(args.engines, engine_dir, args.games, args.time, results_dir, use_opening_book)
    else:
        if len(args.engines) != 2:
            print("Error: Match mode requires exactly 2 engines")
            sys.exit(1)

        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        pgn_file = results_dir / f"{args.engines[0]}_vs_{args.engines[1]}_{timestamp}.pgn"
        run_match(args.engines[0], args.engines[1], engine_dir,
                  args.games, args.time, pgn_file, use_opening_book)


if __name__ == "__main__":
    main()

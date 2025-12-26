#!/usr/bin/env python3
"""
Engine vs Engine competition harness using python-chess.

Usage:
    # Single match between two engines
    ./scripts/compete.py v1-baseline v2-weak-queen --games 100 --time 1.0

    # League (round-robin) between multiple engines
    ./scripts/compete.py v1-baseline v2-weak-queen v3-test --league --games 10 --time 0.5

    # Quick test with fast time control
    ./scripts/compete.py v1-baseline v2-weak-queen --games 10 --time 0.2
"""

import argparse
import math
import os
import sys
import chess
import chess.engine
import chess.pgn
from datetime import datetime
from pathlib import Path
from itertools import combinations


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
              time_per_move: float) -> tuple[str, chess.pgn.Game]:
    """Play a single game and return (result, pgn_game)."""
    board = chess.Board()

    engine1 = chess.engine.SimpleEngine.popen_uci(str(engine1_path))
    engine2 = chess.engine.SimpleEngine.popen_uci(str(engine2_path))

    engines = [engine1, engine2]
    names = [engine1_name, engine2_name]

    game = chess.pgn.Game()
    game.headers["Event"] = "Engine Match"
    game.headers["Date"] = datetime.now().strftime("%Y.%m.%d")
    game.headers["White"] = engine1_name
    game.headers["Black"] = engine2_name

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
              num_games: int, time_per_move: float, pgn_file: Path) -> dict:
    """Run a match between two engines, alternating colors."""

    engine1_path = get_engine_path(engine1_name, engine_dir)
    engine2_path = get_engine_path(engine2_name, engine_dir)

    results = {"1-0": 0, "0-1": 0, "1/2-1/2": 0, "*": 0}
    engine1_points = 0.0
    engine2_points = 0.0

    print(f"\n{'='*60}")
    print(f"Match: {engine1_name} vs {engine2_name}")
    print(f"Games: {num_games}, Time: {time_per_move}s/move")
    print(f"{'='*60}\n")

    games = []

    for i in range(num_games):
        # Alternate colors
        if i % 2 == 0:
            white, black = engine1_name, engine2_name
            white_path, black_path = engine1_path, engine2_path
            is_engine1_white = True
        else:
            white, black = engine2_name, engine1_name
            white_path, black_path = engine2_path, engine1_path
            is_engine1_white = False

        result, game = play_game(white_path, black_path, white, black, time_per_move)
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
        print(f"Game {game_num:3d}/{num_games}: {white} vs {black} -> {result}  "
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
               games_per_pairing: int, time_per_move: float, results_dir: Path):
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
                                  games_per_pairing, time_per_move, pgn_file)
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

    args = parser.parse_args()

    script_dir = Path(__file__).parent
    engine_dir = script_dir.parent / "engines"
    results_dir = script_dir.parent / "results" / "competitions"
    results_dir.mkdir(parents=True, exist_ok=True)

    if args.league:
        if len(args.engines) < 3:
            print("Error: League mode requires at least 3 engines")
            sys.exit(1)
        run_league(args.engines, engine_dir, args.games, args.time, results_dir)
    else:
        if len(args.engines) != 2:
            print("Error: Match mode requires exactly 2 engines")
            sys.exit(1)

        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        pgn_file = results_dir / f"{args.engines[0]}_vs_{args.engines[1]}_{timestamp}.pgn"
        run_match(args.engines[0], args.engines[1], engine_dir,
                  args.games, args.time, pgn_file)


if __name__ == "__main__":
    main()

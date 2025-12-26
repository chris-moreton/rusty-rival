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
import os
import sys
import chess
import chess.engine
import chess.pgn
from datetime import datetime
from pathlib import Path
from itertools import combinations


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
              f"| Score: {engine1_name} {engine1_points:.1f} - {engine2_points:.1f} {engine2_name}")

    # Save PGN
    with open(pgn_file, "w") as f:
        for game in games:
            print(game, file=f)
            print(file=f)

    # Print summary
    print(f"\n{'='*60}")
    print("FINAL RESULTS")
    print(f"{'='*60}")
    print(f"{engine1_name}: {engine1_points:.1f} points")
    print(f"{engine2_name}: {engine2_points:.1f} points")
    print(f"\nResults breakdown: +{results['1-0']} -{results['0-1']} ={results['1/2-1/2']}")
    print(f"PGN saved to: {pgn_file}")
    print(f"{'='*60}\n")

    return {
        engine1_name: engine1_points,
        engine2_name: engine2_points,
        "results": results
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

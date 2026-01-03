#!/usr/bin/env python3
"""
Engine vs Engine competition harness using python-chess.

Usage:
    # Single match between two engines (uses opening book by default)
    # Engine names can be shorthand (v1) or full (v1-baseline)
    ./scripts/compete.py v1 v2 --games 100 --time 1.0

    # Round-robin between 3+ engines (interleaved format)
    # Shows league table with Elo estimates after each round
    ./scripts/compete.py v1 v2 v10 v11 --games 250 --time 0.25

    # Disable opening book (start from initial position)
    ./scripts/compete.py v1 v2 --games 10 --no-book

    # Gauntlet mode: test one engine against all others in engines directory
    # Plays N rounds (2 games per opponent per round) with random openings
    ./scripts/compete.py v20 --gauntlet --games 50 --time 0.5

    # Random mode: randomly pair engines for 2-game matches
    # Each match plays 1 game as white, 1 as black with random openings
    ./scripts/compete.py --random --games 100 --time 0.5

    # Random mode with time range: time randomly selected per match
    ./scripts/compete.py --random --games 100 --timelow 0.1 --timehigh 1.0

    # EPD mode: play through all positions from an EPD file
    # Each position played twice per pairing (once per side)
    ./scripts/compete.py v27 v24 --epd eet.epd --time 1.0
    ./scripts/compete.py v27 v24 v1 --epd engines/epd/pet.epd --time 0.5
"""

import argparse
import json
import math
import random
import subprocess
import sys
import chess
import chess.engine
import chess.pgn
from datetime import datetime
from pathlib import Path
from itertools import combinations


# Default K-factor for Elo updates (higher = faster adjustment)
# Use higher K for provisional ratings (< 30 games)
K_FACTOR_PROVISIONAL = 40
K_FACTOR_ESTABLISHED = 20
PROVISIONAL_GAMES = 30
DEFAULT_ELO = 1500


def get_elo_file_path() -> Path:
    """Get the path to the Elo ratings file."""
    script_dir = Path(__file__).parent
    return script_dir.parent / "results" / "elo_ratings.json"


def load_elo_ratings() -> dict:
    """
    Load Elo ratings from JSON file.
    Returns dict: {engine_name: {"elo": float, "games": int}}
    """
    elo_file = get_elo_file_path()
    if elo_file.exists():
        with open(elo_file, "r") as f:
            return json.load(f)
    return {}


def save_elo_ratings(ratings: dict):
    """Save Elo ratings to JSON file."""
    elo_file = get_elo_file_path()
    elo_file.parent.mkdir(parents=True, exist_ok=True)
    with open(elo_file, "w") as f:
        json.dump(ratings, f, indent=2, sort_keys=True)


def get_engines_config_path() -> Path:
    """Get the path to the engines config file."""
    script_dir = Path(__file__).parent
    return script_dir.parent / "engines" / "engines.json"


def load_engines_config() -> dict:
    """
    Load engine configuration from JSON file.
    Returns dict: {engine_name: {"binary": str, "uci_options": dict (optional)}}
    """
    config_file = get_engines_config_path()
    if config_file.exists():
        with open(config_file, "r") as f:
            return json.load(f)
    return {}


def save_engines_config(config: dict):
    """Save engine configuration to JSON file."""
    config_file = get_engines_config_path()
    config_file.parent.mkdir(parents=True, exist_ok=True)
    with open(config_file, "w") as f:
        json.dump(config, f, indent=2, sort_keys=True)


def load_epd_positions(epd_file: Path) -> list[tuple[str, str]]:
    """
    Load positions from an EPD file.
    Returns list of (fen, position_id) tuples.

    EPD format: FEN [operations]
    Example: 8/8/p2p3p/3k2p1/PP6/3K1P1P/8/8 b - - bm Kc6; id "E_E_T 001";

    We extract the FEN (first 4 fields) and the id if present.
    If no id, we use the position number.
    """
    positions = []
    with open(epd_file, "r") as f:
        for line_num, line in enumerate(f, 1):
            line = line.strip()
            if not line or line.startswith("#"):
                continue

            # EPD has 4 FEN fields (board, side, castling, en passant)
            # followed by optional operations like bm, id, etc.
            parts = line.split()
            if len(parts) < 4:
                continue

            # Construct FEN with default halfmove/fullmove counts
            fen = f"{parts[0]} {parts[1]} {parts[2]} {parts[3]} 0 1"

            # Try to extract id from the line
            pos_id = None
            if 'id "' in line:
                try:
                    start = line.index('id "') + 4
                    end = line.index('"', start)
                    pos_id = line[start:end]
                except ValueError:
                    pass

            if pos_id is None:
                pos_id = f"Position {line_num}"

            positions.append((fen, pos_id))

    return positions


def get_all_engines(engine_dir: Path) -> list[str]:
    """
    Get list of all available engines from the config file.
    Returns sorted list of engine names.
    """
    config = load_engines_config()
    return sorted(config.keys())


def get_active_engines(engine_dir: Path) -> list[str]:
    """
    Get list of active engines from the config file.
    Only returns engines where 'active' is True or not specified.
    Returns sorted list of engine names.
    """
    config = load_engines_config()
    return sorted([name for name, cfg in config.items() if cfg.get("active", True)])


def get_engine_info(name: str, engine_dir: Path) -> tuple[Path, dict]:
    """
    Get engine binary path and UCI options from config.
    Returns (binary_path, uci_options_dict).
    """
    config = load_engines_config()

    if name not in config:
        print(f"Error: Engine '{name}' not found in {get_engines_config_path()}")
        print(f"Available engines: {', '.join(get_all_engines(engine_dir))}")
        sys.exit(1)

    engine_config = config[name]
    binary = engine_config.get("binary", "")

    # Handle relative vs absolute paths
    if binary.startswith("/"):
        binary_path = Path(binary)
    else:
        binary_path = engine_dir / binary

    uci_options = engine_config.get("uci_options", {})

    if not binary_path.exists():
        print(f"Error: Engine binary not found: {binary_path}")
        sys.exit(1)

    return binary_path, uci_options


def get_k_factor(games_played: int) -> float:
    """Get K-factor based on number of games played."""
    if games_played < PROVISIONAL_GAMES:
        return K_FACTOR_PROVISIONAL
    return K_FACTOR_ESTABLISHED


def update_elo_after_game(ratings: dict, white: str, black: str, result: str):
    """
    Update Elo ratings after a single game using the standard Elo formula.

    Formula:
        Expected = 1 / (1 + 10^((opponent_elo - self_elo) / 400))
        New_elo = old_elo + K * (actual_score - expected)

    Modifies ratings dict in place and saves to file.
    """
    # Ensure both players exist in ratings
    for player in [white, black]:
        if player not in ratings:
            # New player gets average of existing ratings or default
            if ratings:
                avg_elo = sum(r["elo"] for r in ratings.values()) / len(ratings)
            else:
                avg_elo = DEFAULT_ELO
            ratings[player] = {"elo": avg_elo, "games": 0}

    white_elo = ratings[white]["elo"]
    black_elo = ratings[black]["elo"]
    white_games = ratings[white]["games"]
    black_games = ratings[black]["games"]

    # Calculate expected scores
    white_expected = 1 / (1 + 10 ** ((black_elo - white_elo) / 400))
    black_expected = 1 - white_expected

    # Actual scores
    if result == "1-0":
        white_actual, black_actual = 1.0, 0.0
    elif result == "0-1":
        white_actual, black_actual = 0.0, 1.0
    elif result == "1/2-1/2":
        white_actual, black_actual = 0.5, 0.5
    else:
        # Unknown result, don't update
        return

    # Get K-factors
    white_k = get_k_factor(white_games)
    black_k = get_k_factor(black_games)

    # Update ratings
    ratings[white]["elo"] += white_k * (white_actual - white_expected)
    ratings[black]["elo"] += black_k * (black_actual - black_expected)
    ratings[white]["games"] += 1
    ratings[black]["games"] += 1

    # Save immediately (crash-safe)
    save_elo_ratings(ratings)


# Opening positions (FEN) - balanced positions after 4-8 moves from various openings
# Each position will be played twice (once with each engine as white)
# 250 positions = 500 unique games maximum
OPENING_BOOK = [
    # ============ SICILIAN DEFENSE (25 variations) ============
    ("rnbqkb1r/pp2pppp/3p1n2/8/3NP3/2N5/PPP2PPP/R1BQKB1R b KQkq - 1 5", "Sicilian Open"),
    ("r1bqkb1r/pp1ppppp/2n2n2/8/3NP3/8/PPP2PPP/RNBQKB1R w KQkq - 4 5", "Sicilian Nc6"),
    ("rnbqkb1r/pp2pppp/3p1n2/8/3NP3/8/PPP1BPPP/RNBQK2R b KQkq - 1 5", "Sicilian Najdorf Be2"),
    ("r1bqkb1r/pp2pppp/2np1n2/8/3NP3/2N5/PPP2PPP/R1BQKB1R w KQkq - 0 6", "Sicilian Scheveningen"),
    ("r1bqkb1r/pp3ppp/2nppn2/8/3NP3/2N1B3/PPP2PPP/R2QKB1R w KQkq - 0 7", "Sicilian Dragon"),
    ("rnbqkb1r/1p2pppp/p2p1n2/8/3NP3/2N5/PPP2PPP/R1BQKB1R w KQkq - 0 6", "Sicilian Najdorf"),
    ("r1bqkb1r/pp2pppp/2np1n2/6B1/3NP3/2N5/PPP2PPP/R2QKB1R b KQkq - 5 6", "Sicilian Richter-Rauzer"),
    ("r1b1kb1r/pp2pppp/1qnp1n2/8/3NP3/2N5/PPP1BPPP/R1BQK2R w KQkq - 4 7", "Sicilian Sozin"),
    ("r1bqk2r/pp2bppp/2nppn2/8/3NP3/2N1B3/PPP1BPPP/R2QK2R b KQkq - 2 8", "Sicilian English Attack"),
    ("r1bqkb1r/5ppp/p1np1n2/1p2p3/4P3/N1N5/PPP1BPPP/R1BQK2R w KQkq - 0 9", "Sicilian Sveshnikov"),
    ("r1bqk2r/pp2bppp/2nppn2/8/4P3/1NN5/PPP1BPPP/R1BQK2R b KQkq - 1 8", "Sicilian Maroczy Bind"),
    ("r1bqkb1r/pp3ppp/2nppn2/8/3NP3/2N5/PPP1BPPP/R1BQK2R w KQkq - 0 7", "Sicilian Classical"),
    ("r1bqkb1r/1p2pppp/p1np1n2/8/3NP3/2N1B3/PPP2PPP/R2QKB1R w KQkq - 0 7", "Sicilian Najdorf Be3"),
    ("r1bqk2r/pp2bppp/2nppn2/8/3NP3/2N1BP2/PPP3PP/R2QKB1R b KQkq - 0 8", "Sicilian Dragon Yugoslav"),
    ("r1b1kb1r/pp3ppp/1qnppn2/8/3NP3/2N1B3/PPP1BPPP/R2QK2R w KQkq - 2 8", "Sicilian Taimanov"),
    ("rnbqkb1r/pp3ppp/3ppn2/8/3NP3/2N5/PPP2PPP/R1BQKB1R w KQkq - 0 6", "Sicilian Kan"),
    ("r1bqkb1r/pp2pppp/2np1n2/8/2BNP3/2N5/PPP2PPP/R1BQK2R b KQkq - 5 6", "Sicilian Bc4"),
    ("rnbqkb1r/pp2pppp/3p1n2/8/2BNP3/8/PPP2PPP/RNBQK2R b KQkq - 1 5", "Sicilian Open Bc4"),
    ("r1bqk2r/pp1nbppp/2npp3/8/3NP3/2N1B3/PPP1BPPP/R2QK2R w KQkq - 2 8", "Sicilian Paulsen"),
    ("r1bqkb1r/pp3ppp/2nppn2/1B6/3NP3/2N5/PPP2PPP/R1BQK2R b KQkq - 1 7", "Sicilian Bb5+"),
    ("rnbqkb1r/pp2pppp/3p1n2/2p5/3PP3/5N2/PPP2PPP/RNBQKB1R w KQkq - 0 4", "Sicilian c5"),
    ("r1bqkb1r/pp3ppp/2nppn2/8/3NP3/2N2P2/PPP3PP/R1BQKB1R b KQkq - 0 7", "Sicilian f3"),
    ("r1bqkb1r/pp2pp1p/2np1np1/8/3NP3/2N1B3/PPP2PPP/R2QKB1R w KQkq - 0 7", "Sicilian Accelerated Dragon"),
    ("rnbqkb1r/pp2pp1p/3p1np1/8/3NP3/2N5/PPP2PPP/R1BQKB1R w KQkq - 0 6", "Sicilian Dragon Setup"),
    ("r1bqk2r/pp2bppp/2np1n2/4p3/3NP3/2N1B3/PPP1BPPP/R2QK2R w KQkq - 0 8", "Sicilian Boleslavsky"),

    # ============ ITALIAN GAME / GIUOCO PIANO (15 variations) ============
    ("r1bqk1nr/pppp1ppp/2n5/2b1p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 4 4", "Italian Game"),
    ("r1bqkb1r/pppp1ppp/2n2n2/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 4 4", "Two Knights"),
    ("r1bqk2r/pppp1ppp/2n2n2/2b1p3/2B1P3/3P1N2/PPP2PPP/RNBQK2R b KQkq - 0 5", "Italian Giuoco Piano"),
    ("r1bqk2r/pppp1ppp/2n2n2/2b1p3/2BPP3/5N2/PPP2PPP/RNBQK2R b KQkq - 0 5", "Italian d4"),
    ("r1bqk2r/pppp1ppp/2n2n2/2b1p3/2B1P3/2NP1N2/PPP2PPP/R1BQK2R b KQkq - 2 5", "Italian Nc3"),
    ("r1bq1rk1/pppp1ppp/2n2n2/2b1p3/2B1P3/3P1N2/PPP2PPP/RNBQ1RK1 w - - 4 7", "Italian Quiet"),
    ("r1bqk2r/pppp1ppp/2n2n2/4p3/1bB1P3/2NP1N2/PPP2PPP/R1BQK2R b KQkq - 2 5", "Italian Evans Gambit"),
    ("r1bqkb1r/pppp1Npp/2n2n2/4p3/2B1P3/8/PPPP1PPP/RNBQK2R b KQkq - 0 4", "Italian Fried Liver"),
    ("r1bqk2r/ppp2ppp/2np1n2/2b1p3/2B1P3/3P1N2/PPP2PPP/RNBQ1RK1 b kq - 0 6", "Italian d6"),
    ("r1bqk2r/pppp1ppp/2n2n2/2b1p3/2B1P3/5N2/PPPP1PPP/RNBQ1RK1 b kq - 5 5", "Italian O-O"),
    ("r1bq1rk1/pppp1ppp/2n2n2/2b1p3/2B1P3/3P1N2/PPP1QPPP/RNB2RK1 b - - 6 7", "Italian Qe2"),
    ("r1bqk2r/ppp2ppp/2np1n2/2b1p3/2B1P3/2PP1N2/PP3PPP/RNBQK2R b KQkq - 0 6", "Italian Main Line"),
    ("r1bqk2r/pppp1ppp/2n2n2/2b1p3/2B1P3/2N2N2/PPPP1PPP/R1BQK2R b KQkq - 5 5", "Four Knights Italian"),
    ("r1bqk2r/pppp1ppp/2n2n2/4p3/1bBPP3/2N2N2/PPP2PPP/R1BQK2R b KQkq - 0 5", "Italian Scotch Gambit"),
    ("r1bq1rk1/ppppbppp/2n2n2/4p3/2B1P3/3P1N2/PPP2PPP/RNBQR1K1 w - - 6 7", "Italian Re1"),

    # ============ RUY LOPEZ (20 variations) ============
    ("r1bqkbnr/pppp1ppp/2n5/1B2p3/4P3/5N2/PPPP1PPP/RNBQK2R b KQkq - 3 3", "Ruy Lopez"),
    ("r1bqkb1r/1ppp1ppp/p1n2n2/1B2p3/4P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 0 5", "Ruy Lopez Morphy"),
    ("r1bqkb1r/1ppp1ppp/p1n2n2/4p3/B3P3/5N2/PPPP1PPP/RNBQK2R b KQkq - 1 5", "Ruy Lopez Closed"),
    ("r1bqkb1r/2pp1ppp/p1n2n2/1p2p3/4P3/1B3N2/PPPP1PPP/RNBQK2R w KQkq - 0 6", "Ruy Lopez Marshall"),
    ("r1bqk2r/2ppbppp/p1n2n2/1p2p3/4P3/1B3N2/PPPP1PPP/RNBQ1RK1 w kq - 2 7", "Ruy Lopez Be7"),
    ("r1bqkb1r/1ppp1ppp/p1n2n2/4p3/B3P3/5N2/PPPP1PPP/RNBQ1RK1 b kq - 2 5", "Ruy Lopez O-O"),
    ("r1bqk2r/1pppbppp/p1n2n2/4p3/B3P3/5N2/PPPP1PPP/RNBQ1RK1 w kq - 3 6", "Ruy Lopez Main Line"),
    ("r1bqkb1r/2pp1ppp/p1n2n2/1p2p3/4P3/1B3N2/PPPP1PPP/RNBQ1RK1 b kq - 1 6", "Ruy Lopez Archangel"),
    ("r1bqk2r/2ppbppp/p1n2n2/1p2p3/4P3/1BP2N2/PP1P1PPP/RNBQ1RK1 b kq - 0 7", "Ruy Lopez c3"),
    ("r1bq1rk1/2ppbppp/p1n2n2/1p2p3/4P3/1BP2N2/PP1P1PPP/RNBQR1K1 b - - 2 8", "Ruy Lopez Re1"),
    ("r1bqkb1r/1ppp1ppp/p1B2n2/4p3/4P3/5N2/PPPP1PPP/RNBQK2R b KQkq - 0 5", "Ruy Lopez Exchange"),
    ("r1bqk2r/2ppbppp/p1n2n2/1p2p3/P3P3/1B3N2/1PPP1PPP/RNBQ1RK1 b kq - 0 7", "Ruy Lopez a4"),
    ("r1bqk2r/1pppbppp/p1n2n2/4p3/B3P3/2N2N2/PPPP1PPP/R1BQK2R b KQkq - 5 6", "Ruy Lopez Nc3"),
    ("r1bqk2r/2ppbppp/p1n2n2/1p2p3/4P3/1B1P1N2/PPP2PPP/RNBQ1RK1 b kq - 0 7", "Ruy Lopez d3"),
    ("r1bqk2r/2ppbppp/p1n2n2/1p2p3/4P3/1B3N1P/PPPP1PP1/RNBQ1RK1 b kq - 0 7", "Ruy Lopez h3"),
    ("r2qkb1r/1bpppppp/p1n2n2/1p6/4P3/1B3N2/PPPP1PPP/RNBQ1RK1 w kq - 2 7", "Ruy Lopez Berlin"),
    ("r1bqk2r/2ppbppp/p1n2n2/1p2N3/4P3/1B6/PPPP1PPP/RNBQ1RK1 b kq - 0 7", "Ruy Lopez Ne5"),
    ("r1bq1rk1/2p1bppp/p1np1n2/1p2p3/4P3/1BP2N2/PP1P1PPP/RNBQR1K1 w - - 0 9", "Ruy Lopez d6 Closed"),
    ("r1bqk2r/2ppbppp/p1n2n2/1p2p3/4P3/1B3N2/PPPPQPPP/RNB2RK1 b kq - 3 7", "Ruy Lopez Qe2"),
    ("r1bq1rk1/2ppbppp/p1n2n2/1p2p3/4P3/1B3N2/PPPP1PPP/RNBQR1K1 b - - 2 8", "Ruy Lopez Normal"),

    # ============ QUEEN'S GAMBIT (20 variations) ============
    ("rnbqkb1r/ppp2ppp/4pn2/3p4/2PP4/2N5/PP2PPPP/R1BQKBNR w KQkq - 2 4", "QGD"),
    ("rnbqkb1r/ppp1pppp/5n2/3p4/2PP4/8/PP2PPPP/RNBQKBNR w KQkq - 2 3", "QG Accepted"),
    ("rnbqkb1r/ppp2ppp/4pn2/3p4/2PP4/5N2/PP2PPPP/RNBQKB1R w KQkq - 2 4", "QGD Orthodox"),
    ("rnbqkb1r/p1p2ppp/1p2pn2/3p4/2PP4/5NP1/PP2PP1P/RNBQKB1R w KQkq - 0 5", "QGD Tartakower"),
    ("rnbqkb1r/ppp2ppp/4pn2/3p4/2PP4/2N2N2/PP2PPPP/R1BQKB1R b KQkq - 3 4", "QGD Two Knights"),
    ("rnbqkb1r/ppp1pppp/5n2/3p4/2PP4/2N5/PP2PPPP/R1BQKBNR b KQkq - 2 3", "QGD Nc3"),
    ("rnbqk2r/ppp1bppp/4pn2/3p4/2PP4/2N2N2/PP2PPPP/R1BQKB1R w KQkq - 4 5", "QGD Ragozin"),
    ("rnbqkb1r/p1p2ppp/1p2pn2/3p4/2PP4/2N2N2/PP2PPPP/R1BQKB1R w KQkq - 0 5", "QGD Tartakower Alt"),
    ("rn1qkb1r/ppp1pppp/4bn2/3p4/2PP4/5N2/PP2PPPP/RNBQKB1R w KQkq - 4 4", "QGD Bf5"),
    ("rnbqkb1r/ppp2ppp/4pn2/3p2B1/2PP4/2N5/PP2PPPP/R2QKBNR b KQkq - 3 4", "QGD Bg5"),
    ("rnbqkb1r/pp3ppp/4pn2/2pp4/2PP4/2N2N2/PP2PPPP/R1BQKB1R w KQkq - 0 5", "QGD Tarrasch"),
    ("rnbqkb1r/ppp2ppp/4pn2/3p4/2PP4/6P1/PP2PP1P/RNBQKBNR b KQkq - 0 4", "QGD Catalan"),
    ("rnbqk2r/ppp1bppp/4pn2/3p4/2PP4/5NP1/PP2PP1P/RNBQKB1R w KQkq - 2 5", "QGD Catalan Be7"),
    ("rnbqkb1r/p1pp1ppp/1p2pn2/8/2PP4/2N5/PP2PPPP/R1BQKBNR w KQkq - 0 5", "QGD Queen's Indian"),
    ("rnbqkb1r/pp3ppp/4pn2/2pP4/3P4/2N5/PP2PPPP/R1BQKBNR b KQkq - 0 5", "QGD Semi-Tarrasch"),
    ("rnbqkb1r/ppp2ppp/4pn2/3p4/2PP4/4PN2/PP3PPP/RNBQKB1R b KQkq - 0 4", "QGD e3"),
    ("rnbqkb1r/ppp2p1p/4pnp1/3p4/2PP4/2N2N2/PP2PPPP/R1BQKB1R w KQkq - 0 5", "QGD Schlechter"),
    ("rnbqk2r/ppp1bppp/4pn2/3p4/2PP4/2N5/PP2PPPP/R1BQKBNR w KQkq - 4 5", "QGD Exchange"),
    ("rn1qkb1r/ppp1pppp/4bn2/3p4/2PP4/2N5/PP2PPPP/R1BQKBNR w KQkq - 4 4", "QGD Early Bf5"),
    ("rnbqkb1r/pp3ppp/4pn2/2pp4/2PP4/2N5/PP2PPPP/R1BQKBNR w KQkq c6 0 5", "QGD Tarrasch Main"),

    # ============ KING'S INDIAN DEFENSE (15 variations) ============
    ("rnbqk2r/ppp1ppbp/3p1np1/8/2PPP3/2N5/PP3PPP/R1BQKBNR w KQkq - 0 5", "King's Indian"),
    ("rnbq1rk1/ppp1ppbp/3p1np1/8/2PPP3/2N2N2/PP2BPPP/R1BQK2R b KQ - 3 6", "KID Classical"),
    ("rnbq1rk1/ppp1ppbp/3p1np1/8/2PPP3/2N2N2/PP3PPP/R1BQKB1R w KQ - 2 6", "KID Samisch"),
    ("rnbq1rk1/ppp2pbp/3p1np1/4p3/2PPP3/2N2N2/PP2BPPP/R1BQK2R w KQ - 0 7", "KID Mar del Plata"),
    ("rnbq1rk1/ppp1ppbp/3p1np1/8/2PP4/2N2NP1/PP2PP1P/R1BQKB1R b KQ - 0 6", "KID Fianchetto"),
    ("rnbq1rk1/ppp2pbp/3p1np1/4p3/2PPP3/2N5/PP2BPPP/R1BQK1NR w KQ - 0 7", "KID Four Pawns"),
    ("rnbq1rk1/pp2ppbp/2pp1np1/8/2PPP3/2N2N2/PP2BPPP/R1BQK2R w KQ - 0 7", "KID Petrosian"),
    ("rnbq1rk1/ppp1ppbp/3p1np1/8/2PPP3/2N1BN2/PP3PPP/R2QKB1R b KQ - 4 6", "KID Be3"),
    ("rnbq1rk1/ppp2pbp/3p1np1/8/2PPp3/2N2N2/PP2BPPP/R1BQK2R w KQ - 0 7", "KID e4e5"),
    ("rnbqr1k1/ppp2pbp/3p1np1/4p3/2PPP3/2N2N2/PP2BPPP/R1BQK2R w KQ - 2 8", "KID Re8"),
    ("r1bq1rk1/ppp1ppbp/2np1np1/8/2PPP3/2N2N2/PP2BPPP/R1BQK2R w KQ - 2 7", "KID Nc6"),
    ("rnbq1rk1/ppp2pbp/3p1np1/4p3/2PP4/2N2NP1/PP2PPBP/R1BQK2R b KQ - 0 7", "KID Fianchetto Main"),
    ("rnbq1rk1/pp2ppbp/2pp1np1/8/2PPP3/2N2N2/PP3PPP/R1BQKB1R w KQ - 0 7", "KID Averbakh"),
    ("rnbq1rk1/ppp1ppbp/3p1np1/6B1/2PPP3/2N5/PP3PPP/R2QKBNR b KQ - 3 6", "KID Bg5"),
    ("rnbq1rk1/ppp2pbp/3p1np1/4p3/2PPP3/2N2P2/PP4PP/R1BQKBNR b KQ - 0 7", "KID Samisch f3"),

    # ============ FRENCH DEFENSE (15 variations) ============
    ("rnbqkbnr/ppp2ppp/4p3/3pP3/3P4/8/PPP2PPP/RNBQKBNR b KQkq - 0 3", "French Advance"),
    ("rnbqkb1r/ppp2ppp/4pn2/3p4/3PP3/2N5/PPP2PPP/R1BQKBNR w KQkq - 2 4", "French Nc3"),
    ("rnbqkb1r/ppp2ppp/4pn2/3pP3/3P4/2N5/PPP2PPP/R1BQKBNR b KQkq - 1 4", "French Steinitz"),
    ("rnbqk2r/ppp1bppp/4pn2/3p4/3PP3/2N2N2/PPP2PPP/R1BQKB1R w KQkq - 4 5", "French Classical"),
    ("rnbqkb1r/ppp2ppp/4pn2/3p4/3PP3/8/PPP2PPP/RNBQKBNR w KQkq - 2 4", "French Exchange"),
    ("rnbqk2r/ppp1bppp/4pn2/3pP3/3P4/2N5/PPP2PPP/R1BQKBNR b KQkq - 2 5", "French Steinitz Main"),
    ("rnbqkbnr/ppp2ppp/4p3/3pP3/3P4/5N2/PPP2PPP/RNBQKB1R b KQkq - 1 3", "French Advance Nf3"),
    ("rnbqk2r/ppp1bppp/4pn2/3p4/3PP3/2N5/PPP2PPP/R1BQKBNR w KQkq - 4 5", "French Classical Be7"),
    ("rnbqkb1r/ppp2ppp/4pn2/3pP3/3P2P1/8/PPP2P1P/RNBQKBNR b KQkq - 0 4", "French g4"),
    ("rnbqk2r/ppp2ppp/4pn2/3p4/1b1PP3/2N5/PPP2PPP/R1BQKBNR w KQkq - 3 5", "French Winawer"),
    ("rnbqk1nr/ppp2ppp/4p3/3pP3/1b1P4/2N5/PPP2PPP/R1BQKBNR w KQkq - 2 4", "French Winawer Main"),
    ("rnbqk2r/ppp1bppp/4pn2/3p4/3PP3/2N2P2/PPP3PP/R1BQKBNR b KQkq - 0 5", "French Rubinstein"),
    ("rnbqk2r/ppp1bppp/4pn2/3pP3/3P4/2N2N2/PPP2PPP/R1BQKB1R b KQkq - 3 5", "French Classical Nf3"),
    ("rnbqkb1r/ppp2ppp/4pn2/8/3Pp3/2N5/PPP2PPP/R1BQKBNR w KQkq - 0 5", "French Rubinstein exd5"),
    ("rnbqk2r/ppp1bppp/4p3/3pP2n/3P4/2N5/PPP2PPP/R1BQKBNR w KQkq - 3 5", "French Alekhine-Chatard"),

    # ============ CARO-KANN (12 variations) ============
    ("rnbqkbnr/pp2pppp/2p5/3pP3/3P4/8/PPP2PPP/RNBQKBNR b KQkq - 0 3", "Caro-Kann Advance"),
    ("rnbqkb1r/pp2pppp/2p2n2/3p4/3PP3/2N5/PPP2PPP/R1BQKBNR w KQkq - 2 4", "Caro-Kann Classical"),
    ("rnbqkb1r/pp2pppp/5n2/2pp4/3PP3/2N5/PPP2PPP/R1BQKBNR w KQkq - 0 4", "Caro-Kann Panov"),
    ("rnbqkb1r/pp2pppp/2p2n2/3p4/3PP3/8/PPP2PPP/RNBQKBNR w KQkq - 2 4", "Caro-Kann Main"),
    ("rn1qkbnr/pp2pppp/2p5/3pPb2/3P4/8/PPP2PPP/RNBQKBNR w KQkq - 1 4", "Caro-Kann Advance Bf5"),
    ("rnbqkb1r/pp2pppp/2p2n2/3p4/3PP3/5N2/PPP2PPP/RNBQKB1R b KQkq - 3 4", "Caro-Kann Two Knights"),
    ("rn1qkbnr/pp2pppp/2p5/3pPb2/3P2P1/8/PPP2P1P/RNBQKBNR b KQkq - 0 4", "Caro-Kann g4"),
    ("rnbqkb1r/pp2pppp/2p2n2/8/3Pp3/2N5/PPP2PPP/R1BQKBNR w KQkq - 0 5", "Caro-Kann Nxe4"),
    ("rn1qkbnr/pp2pppp/2p5/3pPb2/3P4/5N2/PPP2PPP/RNBQKB1R b KQkq - 2 4", "Caro-Kann Short"),
    ("rnbqkb1r/pp2pppp/5n2/2pP4/3P4/8/PPP2PPP/RNBQKBNR b KQkq - 0 4", "Caro-Kann Panov Main"),
    ("rnbqkb1r/pp3ppp/2p2n2/3pp3/3PP3/2N5/PPP2PPP/R1BQKBNR w KQkq - 0 5", "Caro-Kann e5"),
    ("rn1qkb1r/pp2pppp/2p2n2/3p4/3PP1b1/2N5/PPP2PPP/R1BQKBNR w KQkq - 3 5", "Caro-Kann Bg4"),

    # ============ ENGLISH OPENING (12 variations) ============
    ("rnbqkbnr/pppp1ppp/8/4p3/2P5/8/PP1PPPPP/RNBQKBNR w KQkq e6 0 2", "English vs e5"),
    ("rnbqkb1r/pppppppp/5n2/8/2P5/5N2/PP1PPPPP/RNBQKB1R b KQkq - 2 2", "English Nf3"),
    ("rnbqkbnr/pp1ppppp/8/2p5/2P5/5N2/PP1PPPPP/RNBQKB1R b KQkq - 1 2", "English Symmetrical"),
    ("rnbqkbnr/pp1ppppp/8/2p5/2P5/2N5/PP1PPPPP/R1BQKBNR b KQkq - 1 2", "English Nc3"),
    ("r1bqkbnr/pppp1ppp/2n5/4p3/2P5/2N5/PP1PPPPP/R1BQKBNR w KQkq - 2 3", "English Nc6"),
    ("rnbqkb1r/pppp1ppp/5n2/4p3/2P5/2N5/PP1PPPPP/R1BQKBNR w KQkq - 2 3", "English Reversed Sicilian"),
    ("rnbqkbnr/ppp1pppp/8/3p4/2P5/8/PP1PPPPP/RNBQKBNR w KQkq d6 0 2", "English vs d5"),
    ("rnbqkbnr/pp1ppppp/8/2p5/2PP4/8/PP2PPPP/RNBQKBNR b KQkq d3 0 2", "English d4"),
    ("r1bqkbnr/pppp1ppp/2n5/4p3/2P5/5N2/PP1PPPPP/RNBQKB1R w KQkq - 2 3", "English Bremen"),
    ("rnbqkb1r/pppp1ppp/5n2/4p3/2P5/5NP1/PP1PPP1P/RNBQKB1R b KQkq - 0 3", "English King's"),
    ("rnbqkbnr/p1pppppp/1p6/8/2P5/8/PP1PPPPP/RNBQKBNR w KQkq - 0 2", "English vs b6"),
    ("rnbqkb1r/pp1ppppp/5n2/2p5/2P5/5N2/PP1PPPPP/RNBQKB1R w KQkq - 2 3", "English Hedgehog"),

    # ============ SLAV DEFENSE (10 variations) ============
    ("rnbqkb1r/pp2pppp/2p2n2/3p4/2PP4/5N2/PP2PPPP/RNBQKB1R w KQkq - 2 4", "Slav Defense"),
    ("rnbqkb1r/p3pppp/2p2n2/1p1p4/2PP4/5N2/PP2PPPP/RNBQKB1R w KQkq - 0 5", "Slav Chebanenko"),
    ("rnbqkb1r/pp2pppp/2p2n2/3p4/2PP4/2N5/PP2PPPP/R1BQKBNR b KQkq - 3 4", "Slav Nc3"),
    ("rn1qkb1r/pp2pppp/2p2n2/3p1b2/2PP4/5N2/PP2PPPP/RNBQKB1R w KQkq - 4 5", "Slav Bf5"),
    ("rnbqkb1r/pp2pppp/2p2n2/8/2pP4/5N2/PP2PPPP/RNBQKB1R w KQkq - 0 5", "Slav Exchange"),
    ("rnbqkb1r/pp2pppp/2p2n2/3p4/2PP4/2N2N2/PP2PPPP/R1BQKB1R b KQkq - 3 4", "Slav Two Knights"),
    ("rnbqkb1r/pp2pppp/2p2n2/3p4/2PP4/4PN2/PP3PPP/RNBQKB1R b KQkq - 0 4", "Slav e3"),
    ("rn1qkb1r/pp2pppp/2p2n2/3p4/2PP2b1/5N2/PP2PPPP/RNBQKB1R w KQkq - 4 5", "Slav Bg4"),
    ("rnbqkb1r/1p2pppp/p1p2n2/3p4/2PP4/5N2/PP2PPPP/RNBQKB1R w KQkq - 0 5", "Slav a6"),
    ("rnbqkb1r/pp3ppp/2p2n2/3pp3/2PP4/5N2/PP2PPPP/RNBQKB1R w KQkq e6 0 5", "Slav Semi-Slav"),

    # ============ NIMZO-INDIAN (12 variations) ============
    ("rnbqk2r/pppp1ppp/4pn2/8/1bPP4/2N5/PP2PPPP/R1BQKBNR w KQkq - 2 4", "Nimzo-Indian"),
    ("rnbqk2r/pppp1ppp/4pn2/8/1bPP4/2N2N2/PP2PPPP/R1BQKB1R b KQkq - 3 4", "Nimzo-Indian Classical"),
    ("rnbqk2r/pppp1ppp/4pn2/8/1bPP4/2N1P3/PP3PPP/R1BQKBNR b KQkq - 0 4", "Nimzo-Indian Rubinstein"),
    ("rnbqk2r/pppp1ppp/4pn2/8/1bPP4/2N5/PPQ1PPPP/R1B1KBNR b KQkq - 3 4", "Nimzo-Indian Qc2"),
    ("rnbqk2r/p1pp1ppp/1p2pn2/8/1bPP4/2N2N2/PP2PPPP/R1BQKB1R w KQkq - 0 5", "Nimzo-Indian b6"),
    ("rnbq1rk1/pppp1ppp/4pn2/8/1bPP4/2N1P3/PP3PPP/R1BQKBNR w KQ - 1 5", "Nimzo-Indian O-O"),
    ("rnbqk2r/pppp1ppp/4pn2/6B1/1bPP4/2N5/PP2PPPP/R2QKBNR b KQkq - 3 4", "Nimzo-Indian Bg5"),
    ("rnbq1rk1/p1pp1ppp/1p2pn2/8/1bPP4/2N1PN2/PP3PPP/R1BQKB1R w KQ - 0 6", "Nimzo-Indian Fischer"),
    ("rnbqk2r/pppp1ppp/4pn2/8/2PP4/P1b5/1P2PPPP/R1BQKBNR w KQkq - 0 5", "Nimzo-Indian Samisch"),
    ("rnbq1rk1/pppp1ppp/4pn2/8/1bPP4/2N2N2/PP2PPPP/R1BQKB1R w KQ - 4 5", "Nimzo-Indian Main"),
    ("rnbqk2r/pppp1ppp/4pn2/8/1bPP4/2N1P3/PP1B1PPP/R2QKBNR b KQkq - 2 5", "Nimzo-Indian Bd2"),
    ("rnbq1rk1/ppp2ppp/4pn2/3p4/1bPP4/2N1PN2/PP3PPP/R1BQKB1R w KQ - 0 6", "Nimzo-Indian d5"),

    # ============ SCOTCH GAME (8 variations) ============
    ("r1bqkbnr/pppp1ppp/2n5/4p3/3PP3/5N2/PPP2PPP/RNBQKB1R b KQkq d3 0 3", "Scotch Game"),
    ("r1bqkb1r/pppp1ppp/2n2n2/4N3/3PP3/8/PPP2PPP/RNBQKB1R b KQkq - 0 4", "Scotch Four Knights"),
    ("r1bqkbnr/pppp1ppp/2n5/8/3NP3/8/PPP2PPP/RNBQKB1R b KQkq - 0 4", "Scotch Main"),
    ("r1bqkb1r/pppp1ppp/2n2n2/8/3NP3/8/PPP2PPP/RNBQKB1R w KQkq - 1 5", "Scotch Nf6"),
    ("r1bqkbnr/pppp1ppp/8/4n3/3PP3/8/PPP2PPP/RNBQKB1R w KQkq - 0 4", "Scotch Nxe4"),
    ("r1bqk1nr/pppp1ppp/2n5/2b5/3NP3/8/PPP2PPP/RNBQKB1R w KQkq - 1 5", "Scotch Classical"),
    ("r1bqkbnr/pppp1ppp/2n5/8/3NP3/2N5/PPP2PPP/R1BQKB1R b KQkq - 2 4", "Scotch Nc3"),
    ("r1bqk2r/pppp1ppp/2n2n2/2b5/3NP3/2N5/PPP2PPP/R1BQKB1R w KQkq - 4 5", "Scotch Four Knights Bc5"),

    # ============ PETROV DEFENSE (8 variations) ============
    ("rnbqkb1r/pppp1ppp/5n2/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 2 3", "Petrov Defense"),
    ("rnbqkb1r/pppp1ppp/8/4p3/4n3/5N2/PPPPQPPP/RNB1KB1R b KQkq - 3 4", "Petrov Classical"),
    ("rnbqkb1r/pppp1ppp/5n2/4N3/4P3/8/PPPP1PPP/RNBQKB1R b KQkq - 0 3", "Petrov Nxe5"),
    ("rnbqkb1r/ppp2ppp/3p1n2/4N3/4P3/8/PPPP1PPP/RNBQKB1R w KQkq - 0 4", "Petrov d6"),
    ("rnbqkb1r/pppp1ppp/8/4p3/3Pn3/5N2/PPP2PPP/RNBQKB1R b KQkq - 0 4", "Petrov d4"),
    ("rnbqkb1r/ppp2ppp/3p1n2/4p3/3PP3/5N2/PPP2PPP/RNBQKB1R b KQkq - 0 4", "Petrov Steinitz"),
    ("rnbqkb1r/pppp1ppp/5n2/4p3/4P3/2N2N2/PPPP1PPP/R1BQKB1R b KQkq - 3 3", "Petrov Three Knights"),
    ("r1bqkb1r/pppp1ppp/2n2n2/4p3/4P3/2N2N2/PPPP1PPP/R1BQKB1R w KQkq - 4 4", "Petrov Four Knights"),

    # ============ PIRC DEFENSE (8 variations) ============
    ("rnbqkb1r/ppp1pppp/3p1n2/8/3PP3/8/PPP2PPP/RNBQKBNR w KQkq - 2 3", "Pirc Defense"),
    ("rnbqkb1r/ppp1pp1p/3p1np1/8/3PP3/2N5/PPP2PPP/R1BQKBNR w KQkq - 0 4", "Pirc Austrian"),
    ("rnbqkb1r/ppp1pppp/3p1n2/8/3PP3/2N5/PPP2PPP/R1BQKBNR b KQkq - 3 3", "Pirc Nc3"),
    ("rnbqkb1r/ppp1pp1p/3p1np1/8/3PP3/2N2N2/PPP2PPP/R1BQKB1R b KQkq - 1 4", "Pirc Classical"),
    ("rnbqkb1r/ppp1pp1p/3p1np1/8/3PPP2/2N5/PPP3PP/R1BQKBNR b KQkq - 0 4", "Pirc Austrian f4"),
    ("rnbqk2r/ppp1ppbp/3p1np1/8/3PP3/2N2N2/PPP2PPP/R1BQKB1R w KQkq - 2 5", "Pirc Classical Bg7"),
    ("rnbqkb1r/ppp1pppp/3p1n2/8/3PP3/5N2/PPP2PPP/RNBQKB1R b KQkq - 3 3", "Pirc Nf3"),
    ("rnbqk2r/ppp1ppbp/3p1np1/8/3PP1P1/2N5/PPP2P1P/R1BQKBNR b KQkq - 0 5", "Pirc g4"),

    # ============ SCANDINAVIAN (6 variations) ============
    ("rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 2", "Scandinavian"),
    ("rnb1kbnr/ppp1pppp/8/3q4/8/2N5/PPPP1PPP/R1BQKBNR b KQkq - 1 3", "Scandinavian Qxd5"),
    ("rnbqkbnr/ppp1pppp/8/8/4p3/2N5/PPPP1PPP/R1BQKBNR b KQkq - 1 3", "Scandinavian Nxd5"),
    ("rnb1kbnr/ppp1pppp/8/3q4/8/5N2/PPPP1PPP/RNBQKB1R b KQkq - 1 3", "Scandinavian Nf3"),
    ("rnb1kbnr/ppp1pppp/3q4/8/8/2N5/PPPP1PPP/R1BQKBNR w KQkq - 2 4", "Scandinavian Qd6"),
    ("rn1qkbnr/ppp1pppp/8/3p1b2/4P3/2N5/PPPP1PPP/R1BQKBNR w KQkq - 2 4", "Scandinavian Bf5"),

    # ============ LONDON SYSTEM (8 variations) ============
    ("rnbqkb1r/ppp1pppp/3p1n2/8/3P1B2/5N2/PPP1PPPP/RN1QKB1R b KQkq - 3 3", "London System"),
    ("rnbqkb1r/ppp1pppp/5n2/3p4/3P1B2/5N2/PPP1PPPP/RN1QKB1R b KQkq - 3 3", "London vs d5"),
    ("rnbqkb1r/ppp1pppp/5n2/3p4/3P1B2/4P3/PPP2PPP/RN1QKBNR b KQkq - 0 3", "London e3"),
    ("rnbqk2r/ppp1ppbp/3p1np1/8/3P1B2/5N2/PPP1PPPP/RN1QKB1R w KQkq - 0 5", "London vs KID"),
    ("rnbqkb1r/ppp2ppp/4pn2/3p4/3P1B2/5N2/PPP1PPPP/RN1QKB1R w KQkq - 0 4", "London vs e6"),
    ("rnbqkb1r/pp2pppp/2p2n2/3p4/3P1B2/5N2/PPP1PPPP/RN1QKB1R w KQkq - 0 4", "London vs Slav"),
    ("rnbqkb1r/ppp1pppp/5n2/3p2B1/3P4/5N2/PPP1PPPP/RN1QKB1R b KQkq - 3 3", "London Torre"),
    ("rnbqk2r/ppp1bppp/4pn2/3p4/3P1B2/4PN2/PPP2PPP/RN1QKB1R w KQkq - 2 5", "London Classical"),

    # ============ CATALAN (8 variations) ============
    ("rnbqkb1r/pppp1ppp/4pn2/8/2PP4/6P1/PP2PP1P/RNBQKBNR b KQkq - 0 3", "Catalan"),
    ("rnbqkb1r/ppp2ppp/4pn2/3p4/2PP4/6P1/PP2PP1P/RNBQKBNR w KQkq - 0 4", "Catalan Open"),
    ("rnbqk2r/ppp1bppp/4pn2/3p4/2PP4/6P1/PP2PPBP/RNBQK1NR w KQkq - 2 5", "Catalan Be7"),
    ("rnbqkb1r/ppp2ppp/4pn2/8/2pP4/6P1/PP2PPBP/RNBQK1NR w KQkq - 0 5", "Catalan dxc4"),
    ("rnbqk2r/ppp1bppp/4pn2/3p4/2PP4/5NP1/PP2PPBP/RNBQK2R b KQkq - 3 5", "Catalan Main"),
    ("rnbqkb1r/p1p2ppp/1p2pn2/3p4/2PP4/6P1/PP2PPBP/RNBQK1NR w KQkq - 0 5", "Catalan b6"),
    ("rnbqk2r/ppp2ppp/4pn2/3p4/1bPP4/6P1/PP2PPBP/RNBQK1NR w KQkq - 2 5", "Catalan Bb4"),
    ("rnbq1rk1/ppp1bppp/4pn2/3p4/2PP4/5NP1/PP2PPBP/RNBQK2R w KQ - 4 6", "Catalan Closed"),

    # ============ GRUNFELD (8 variations) ============
    ("rnbqkb1r/ppp1pp1p/5np1/3p4/2PP4/2N5/PP2PPPP/R1BQKBNR w KQkq d6 0 4", "Grunfeld"),
    ("rnbqkb1r/ppp1pp1p/6p1/3n4/3P4/2N5/PP2PPPP/R1BQKBNR w KQkq - 0 5", "Grunfeld Exchange"),
    ("rnbqkb1r/ppp1pp1p/5np1/3p4/2PP4/1QN5/PP2PPPP/R1B1KBNR b KQkq - 3 4", "Grunfeld Russian"),
    ("rnbqkb1r/ppp1pp1p/5np1/3p4/2PP4/2N2N2/PP2PPPP/R1BQKB1R b KQkq - 3 4", "Grunfeld Two Knights"),
    ("rnbqk2r/ppp1ppbp/5np1/3p4/2PP4/2N2N2/PP2PPPP/R1BQKB1R w KQkq - 2 5", "Grunfeld Bg7"),
    ("rnbqkb1r/ppp1pp1p/6p1/3n4/3PP3/2N5/PP3PPP/R1BQKBNR b KQkq - 0 5", "Grunfeld Exchange e4"),
    ("rnbqk2r/ppp1ppbp/5np1/8/2pP4/2N2NP1/PP2PP1P/R1BQKB1R w KQkq - 0 6", "Grunfeld Fianchetto"),
    ("rnbqkb1r/ppp1pp1p/5np1/3p4/2PP1B2/2N5/PP2PPPP/R2QKBNR b KQkq - 3 4", "Grunfeld Bf4"),

    # ============ VIENNA GAME (6 variations) ============
    ("rnbqkb1r/pppp1ppp/5n2/4p3/4P3/2N5/PPPP1PPP/R1BQKBNR w KQkq - 2 3", "Vienna Game"),
    ("rnbqkb1r/pppp1ppp/5n2/4p3/4PP2/2N5/PPPP2PP/R1BQKBNR b KQkq - 0 3", "Vienna Gambit"),
    ("rnbqkb1r/pppp1ppp/5n2/4p3/2B1P3/2N5/PPPP1PPP/R1BQK1NR b KQkq - 3 3", "Vienna Bc4"),
    ("rnbqkb1r/pppp1ppp/8/4p3/4Pn2/2N5/PPPP1PPP/R1BQKBNR w KQkq - 0 4", "Vienna Nxe4"),
    ("r1bqkb1r/pppp1ppp/2n2n2/4p3/4P3/2N2N2/PPPP1PPP/R1BQKB1R w KQkq - 4 4", "Vienna Four Knights"),
    ("rnbqkb1r/pppp1ppp/5n2/4p3/4P3/2N3P1/PPPP1P1P/R1BQKBNR b KQkq - 0 3", "Vienna g3"),

    # ============ KING'S GAMBIT (6 variations) ============
    ("rnbqkbnr/pppp1ppp/8/4p3/4PP2/8/PPPP2PP/RNBQKBNR b KQkq f3 0 2", "King's Gambit"),
    ("rnbqkbnr/pppp1ppp/8/8/4Pp2/8/PPPP2PP/RNBQKBNR w KQkq - 0 3", "King's Gambit Accepted"),
    ("rnbqkbnr/pppp1ppp/8/4p3/4PP2/8/PPPP2PP/RNBQKBNR b KQkq - 0 2", "King's Gambit Main"),
    ("rnbqkbnr/pppp1ppp/8/8/4Pp2/5N2/PPPP2PP/RNBQKB1R b KQkq - 1 3", "King's Gambit Nf3"),
    ("rnbqkbnr/ppp2ppp/8/3pp3/4PP2/8/PPPP2PP/RNBQKBNR w KQkq d6 0 3", "King's Gambit Declined"),
    ("rnbqk1nr/pppp1ppp/8/2b1p3/4PP2/8/PPPP2PP/RNBQKBNR w KQkq - 1 3", "King's Gambit Classical"),

    # ============ MODERN DEFENSE (6 variations) ============
    ("rnbqkbnr/pppppp1p/6p1/8/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2", "Modern Defense"),
    ("rnbqkbnr/pppppp1p/6p1/8/3PP3/8/PPP2PPP/RNBQKBNR b KQkq - 0 2", "Modern d4"),
    ("rnbqk1nr/ppppppbp/6p1/8/3PP3/2N5/PPP2PPP/R1BQKBNR b KQkq - 2 3", "Modern Averbakh"),
    ("rnbqk1nr/ppppppbp/6p1/8/3PP3/8/PPP2PPP/RNBQKBNR w KQkq - 1 3", "Modern Bg7"),
    ("rnbqk1nr/ppp1ppbp/3p2p1/8/3PP3/2N5/PPP2PPP/R1BQKBNR w KQkq - 0 4", "Modern Pirc-like"),
    ("rnbqk1nr/ppppppbp/6p1/8/3PP3/5N2/PPP2PPP/RNBQKB1R b KQkq - 2 3", "Modern Nf3"),

    # ============ ALEKHINE'S DEFENSE (6 variations) ============
    ("rnbqkb1r/pppppppp/5n2/8/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 1 2", "Alekhine's Defense"),
    ("rnbqkb1r/pppppppp/8/4n3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 3", "Alekhine's e5"),
    ("rnbqkb1r/pppppppp/8/3nP3/8/8/PPPP1PPP/RNBQKBNR b KQkq - 0 3", "Alekhine's Four Pawns"),
    ("rnbqkb1r/ppp1pppp/3p4/3nP3/3P4/8/PPP2PPP/RNBQKBNR b KQkq - 0 4", "Alekhine's Modern"),
    ("rnbqkb1r/ppp1pppp/3p1n2/4P3/3P4/8/PPP2PPP/RNBQKBNR b KQkq - 0 4", "Alekhine's Exchange"),
    ("rnbqkb1r/pppppppp/5n2/4P3/8/8/PPPP1PPP/RNBQKBNR b KQkq - 0 2", "Alekhine's Chase"),

    # ============ BENONI (8 variations) ============
    ("rnbqkb1r/pp1p1ppp/4pn2/2pP4/2P5/8/PP2PPPP/RNBQKBNR w KQkq - 0 4", "Benoni"),
    ("rnbqkb1r/pp1p1ppp/4pn2/2pP4/2P5/2N5/PP2PPPP/R1BQKBNR b KQkq - 1 4", "Modern Benoni"),
    ("rnbqkb1r/pp3ppp/3ppn2/2pP4/2P5/2N5/PP2PPPP/R1BQKBNR w KQkq - 0 5", "Benoni Main"),
    ("rnbqkb1r/pp1p1ppp/4pn2/8/2pP4/2N5/PP2PPPP/R1BQKBNR w KQkq - 0 5", "Benoni Benko"),
    ("rnbqk2r/pp1p1ppp/4pn2/2pP4/2P5/2Nb4/PP2PPPP/R1BQKBNR w KQkq - 2 5", "Benoni Bb4"),
    ("rnbqkb1r/pp3ppp/4pn2/2pP4/8/2N5/PP2PPPP/R1BQKBNR w KQkq - 0 5", "Benoni d5"),
    ("rnbq1rk1/pp1p1ppp/4pn2/2pP4/2P5/2N2N2/PP2PPPP/R1BQKB1R w KQ - 2 6", "Benoni O-O"),
    ("rnbqkb1r/pp1p1ppp/4pn2/2pP4/2P1P3/8/PP3PPP/RNBQKBNR b KQkq - 0 4", "Benoni e4"),

    # ============ DUTCH DEFENSE (6 variations) ============
    ("rnbqkbnr/ppppp1pp/8/5p2/3P4/8/PPP1PPPP/RNBQKBNR w KQkq f6 0 2", "Dutch Defense"),
    ("rnbqkbnr/ppppp1pp/8/5p2/2PP4/8/PP2PPPP/RNBQKBNR b KQkq - 0 2", "Dutch c4"),
    ("rnbqkb1r/ppppp1pp/5n2/5p2/2PP4/6P1/PP2PP1P/RNBQKBNR b KQkq - 0 3", "Dutch Leningrad"),
    ("rnbqkb1r/pppp2pp/4pn2/5p2/2PP4/6P1/PP2PP1P/RNBQKBNR w KQkq - 0 4", "Dutch Stonewall"),
    ("rnbqkb1r/ppppp1pp/5n2/5p2/2PP4/5N2/PP2PPPP/RNBQKB1R b KQkq - 2 3", "Dutch Classical"),
    ("rnbqk2r/ppppp1bp/5np1/5p2/2PP4/6P1/PP2PPBP/RNBQK1NR w KQkq - 2 5", "Dutch Leningrad Main"),

    # ============ MISCELLANEOUS OPENINGS (10 variations) ============
    ("rnbqkbnr/ppp1pppp/8/3p4/8/5NP1/PPPPPP1P/RNBQKB1R b KQkq - 0 2", "Reti Opening"),
    ("rnbqkbnr/pppp1ppp/8/4p3/6P1/8/PPPPPP1P/RNBQKBNR w KQkq e6 0 2", "Grob Attack"),
    ("rnbqkbnr/pppppppp/8/8/3P4/8/PPP1PPPP/RNBQKBNR b KQkq d3 0 1", "Queen's Pawn"),
    ("rnbqkb1r/pppp1ppp/5n2/4p3/2P5/8/PP1PPPPP/RNBQKBNR w KQkq e6 0 3", "English e5"),
    ("rnbqkbnr/pppp1ppp/4p3/8/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2", "French 1...e6"),
    ("r1bqkbnr/pppp1ppp/2n5/4p3/2P5/2N5/PP1PPPPP/R1BQKBNR w KQkq - 2 3", "English e5 Nc6"),
    ("rnbqkbnr/pp1ppppp/8/2p5/4P3/8/PPPP1PPP/RNBQKBNR w KQkq c6 0 2", "Sicilian 1...c5"),
    ("rnbqkb1r/pp1ppppp/5n2/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 2 3", "Sicilian Nf6"),
    ("rnbqkbnr/pp1ppppp/2p5/8/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2", "Caro-Kann 1...c6"),
    ("rnbqkbnr/ppp1pppp/3p4/8/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2", "Pirc 1...d6"),
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


def resolve_engine_name(shorthand: str, engine_dir: Path) -> str:
    """
    Resolve a shorthand engine name to the full directory name.
    E.g., 'v1' -> 'v1-baseline', 'v10' -> 'v10-arrayvec-movelist'
    """
    # If exact match exists, use it
    if (engine_dir / shorthand).exists():
        return shorthand

    # Look for directories starting with the shorthand followed by '-'
    pattern = f"{shorthand}-*"
    matches = list(engine_dir.glob(pattern))

    if len(matches) == 1:
        return matches[0].name
    elif len(matches) == 0:
        print(f"Error: No engine found matching '{shorthand}'")
        print(f"Available engines: {', '.join(sorted(d.name for d in engine_dir.iterdir() if d.is_dir()))}")
        sys.exit(1)
    else:
        print(f"Error: Ambiguous engine name '{shorthand}' matches: {', '.join(m.name for m in matches)}")
        sys.exit(1)


def play_game(engine1_path: Path, engine2_path: Path,
              engine1_name: str, engine2_name: str,
              time_per_move: float,
              start_fen: str = None,
              opening_name: str = None,
              engine1_uci_options: dict = None,
              engine2_uci_options: dict = None) -> tuple[str, chess.pgn.Game]:
    """Play a single game and return (result, pgn_game)."""
    if start_fen:
        board = chess.Board(start_fen)
    else:
        board = chess.Board()

    engine1 = chess.engine.SimpleEngine.popen_uci(str(engine1_path), stderr=subprocess.DEVNULL)
    engine2 = chess.engine.SimpleEngine.popen_uci(str(engine2_path), stderr=subprocess.DEVNULL)

    # Configure UCI options if provided
    if engine1_uci_options:
        engine1.configure(engine1_uci_options)
    if engine2_uci_options:
        engine2.configure(engine2_uci_options)

    engines = [engine1, engine2]

    game = chess.pgn.Game()
    game.headers["Event"] = "Engine Match"
    game.headers["Date"] = datetime.now().strftime("%Y.%m.%d")
    game.headers["White"] = engine1_name
    game.headers["Black"] = engine2_name
    game.headers["TimeControl"] = f"{time_per_move:.2f}s/move"
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

    engine1_path, engine1_uci_options = get_engine_info(engine1_name, engine_dir)
    engine2_path, engine2_uci_options = get_engine_info(engine2_name, engine_dir)

    # Load persistent Elo ratings
    elo_ratings = load_elo_ratings()

    # Initialize engines if needed
    for name in [engine1_name, engine2_name]:
        if name not in elo_ratings:
            if elo_ratings:
                avg_elo = sum(r["elo"] for r in elo_ratings.values()) / len(elo_ratings)
            else:
                avg_elo = DEFAULT_ELO
            elo_ratings[name] = {"elo": avg_elo, "games": 0}
            save_elo_ratings(elo_ratings)

    # Store starting Elo for display
    start_elo = {
        engine1_name: elo_ratings[engine1_name]["elo"],
        engine2_name: elo_ratings[engine2_name]["elo"]
    }

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

    print(f"\n{'='*70}")
    print(f"Match: {engine1_name} vs {engine2_name}")
    print(f"Games: {num_games}, Time: {time_per_move}s/move")
    if use_opening_book:
        print(f"Opening book: {len(OPENING_BOOK)} positions (randomized)")
    else:
        print("Opening book: disabled")
    print(f"PGN output: {pgn_file}")
    print(f"Elo ratings: {get_elo_file_path()}")
    print(f"{'='*70}")

    # Show starting Elo
    print(f"\nStarting Elo:")
    for name in [engine1_name, engine2_name]:
        data = elo_ratings[name]
        prov = "?" if data["games"] < PROVISIONAL_GAMES else ""
        print(f"  {name}: {data['elo']:.0f} ({data['games']} games){prov}")
    print(flush=True)

    # Clear/create the PGN file at the start
    with open(pgn_file, "w") as f:
        pass

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
            white_uci, black_uci = engine1_uci_options, engine2_uci_options
            is_engine1_white = True
        else:
            white, black = engine2_name, engine1_name
            white_path, black_path = engine2_path, engine1_path
            white_uci, black_uci = engine2_uci_options, engine1_uci_options
            is_engine1_white = False

        result, game = play_game(white_path, black_path, white, black,
                                  time_per_move, opening_fen, opening_name,
                                  white_uci, black_uci)
        results[result] += 1

        # Append game to PGN file immediately (crash-safe)
        with open(pgn_file, "a") as f:
            print(game, file=f)
            print(file=f)

        # Update persistent Elo ratings
        update_elo_after_game(elo_ratings, white, black, result)

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

    # Calculate Elo difference for this match
    # From engine1's perspective: wins are when engine1 won
    engine1_wins = int(engine1_points - (results["1/2-1/2"] * 0.5))
    engine1_losses = int(engine2_points - (results["1/2-1/2"] * 0.5))
    elo_diff, elo_error = calculate_elo_difference(
        engine1_wins, engine1_losses, results["1/2-1/2"]
    )

    # Print summary
    print(f"\n{'='*70}")
    print("FINAL RESULTS")
    print(f"{'='*70}")
    print(f"{engine1_name}: {engine1_points:.1f} points")
    print(f"{engine2_name}: {engine2_points:.1f} points")
    print(f"\nResults: +{engine1_wins} -{engine1_losses} ={results['1/2-1/2']}")
    win_rate = engine1_points / num_games * 100
    print(f"Win rate: {win_rate:.1f}%")

    if elo_diff > 0:
        print(f"\nMatch Elo difference: {engine1_name} is +{elo_diff:.0f} (±{elo_error:.0f}) stronger")
    else:
        print(f"\nMatch Elo difference: {engine2_name} is +{-elo_diff:.0f} (±{elo_error:.0f}) stronger")

    # Show updated Elo ratings
    print(f"\nUpdated Elo ratings:")
    for name in [engine1_name, engine2_name]:
        data = elo_ratings[name]
        delta = data["elo"] - start_elo[name]
        prov = "?" if data["games"] < PROVISIONAL_GAMES else ""
        print(f"  {name}: {data['elo']:.0f} ({delta:+.0f}) - {data['games']} games{prov}")

    print(f"\nPGN saved to: {pgn_file}")
    print(f"Elo ratings saved to: {get_elo_file_path()}")
    print(f"{'='*70}\n")

    return {
        engine1_name: engine1_points,
        engine2_name: engine2_points,
        "results": results,
        "elo_diff": elo_diff,
        "elo_error": elo_error
    }


def print_league_table(ratings: dict, competitors: set[str], games_this_comp: dict[str, int],
                       points_this_comp: dict[str, float], round_num: int, is_final: bool = False,
                       competitors_only: bool = False, game_num: int = 0, total_games: int = 0):
    """
    Print the league standings.

    Args:
        ratings: Global Elo ratings dict {engine: {"elo": float, "games": int}}
        competitors: Set of engine names in the current competition
        games_this_comp: Dict of games played this competition per engine
        points_this_comp: Dict of points scored this competition per engine
        round_num: Current round number
        is_final: Whether this is the final standings
        competitors_only: If True, only show engines in current competition
        game_num: Current game number (for per-game display)
        total_games: Total games in competition (for per-game display)
    """
    if competitors_only:
        # Compact table showing only competitors, sorted by points
        sorted_competitors = sorted(
            [(name, points_this_comp.get(name, 0), ratings[name]["elo"]) for name in competitors],
            key=lambda x: (-x[1], -x[2])  # Sort by points desc, then Elo desc
        )

        print(f"\n{'='*60}")
        print(f"STANDINGS ({game_num}/{total_games} games)")
        print(f"{'='*60}")
        print(f"{'#':<4}{'Engine':<28}{'Points':<10}{'Elo':<10}")
        print(f"{'-'*60}")

        for rank, (name, points, elo) in enumerate(sorted_competitors, 1):
            comp_games = games_this_comp.get(name, 0)
            prov = "?" if ratings[name]["games"] < PROVISIONAL_GAMES else ""
            print(f"{rank:<4}{name:<28}{points:<10.1f}{elo:<10.0f}{prov}")

        print(f"{'='*60}")
        return

    # Full table (original behavior)
    header = "FINAL LEAGUE STANDINGS" if is_final else f"STANDINGS AFTER ROUND {round_num}"
    print(f"\n{'='*95}")
    print(header)
    print(f"{'='*95}")
    print(f"{'Elo#':<6}{'Engine':<28}{'Elo':<10}{'Comp#':<7}{'Points':<10}{'Games':<8}{'Total':<8}{'Status':<10}")
    print(f"{'-'*95}")

    # Sort all engines by Elo (descending)
    sorted_engines = sorted(ratings.items(), key=lambda x: -x[1]["elo"])

    # Calculate competition rankings (by points)
    comp_rankings = {}
    if competitors:
        sorted_by_points = sorted(
            [(name, points_this_comp.get(name, 0)) for name in competitors],
            key=lambda x: -x[1]
        )
        for rank, (name, _) in enumerate(sorted_by_points, 1):
            comp_rankings[name] = rank

    for elo_rank, (name, data) in enumerate(sorted_engines, 1):
        elo = data["elo"]
        total_games = data["games"]
        comp_games = games_this_comp.get(name, 0)
        points = points_this_comp.get(name, 0)

        # Highlight current competitors
        if name in competitors:
            marker = "*"
            comp_rank = str(comp_rankings[name])
            status = "playing" if not is_final else "done"
        else:
            marker = " "
            comp_rank = "-"
            status = ""
            points = "-"

        # Mark provisional ratings
        if total_games < PROVISIONAL_GAMES:
            prov = "?"
        else:
            prov = ""

        display_name = f"{marker}{name}"
        points_str = f"{points:.1f}" if isinstance(points, float) else points
        print(f"{elo_rank:<6}{display_name:<28}{elo:<10.0f}{comp_rank:<7}{points_str:<10}{comp_games:<8}{total_games:<8}{status}{prov}")

    print(f"{'='*95}")
    print(f"  Elo# = overall ranking, Comp# = current competition ranking")
    print(f"  * = in current competition, ? = provisional rating (<{PROVISIONAL_GAMES} games)")
    print()


def run_epd(engine_names: list[str], engine_dir: Path, epd_file: Path,
            time_per_move: float, results_dir: Path):
    """
    Run a competition using positions from an EPD file.
    Each position is played twice (once with each engine as white).
    For 2 engines, it's head-to-head. For 3+ engines, it's round-robin per position.
    """
    from itertools import combinations

    # Load EPD positions
    positions = load_epd_positions(epd_file)
    if not positions:
        print(f"Error: No valid positions found in {epd_file}")
        sys.exit(1)

    print(f"\n{'='*70}")
    print(f"EPD Competition: {epd_file.name}")
    print(f"Positions: {len(positions)}")
    print(f"Engines: {', '.join(engine_names)}")
    print(f"Time: {time_per_move}s/move")
    print(f"Note: Elo ratings NOT updated (EPD mode uses specialized positions)")
    print(f"{'='*70}")

    # Get engine info
    engine_info = {}
    for name in engine_names:
        path, uci_options = get_engine_info(name, engine_dir)
        engine_info[name] = {"path": path, "uci_options": uci_options}

    # Load persistent Elo ratings
    elo_ratings = load_elo_ratings()

    # Initialize engines if needed
    for name in engine_names:
        if name not in elo_ratings:
            if elo_ratings:
                avg_elo = sum(r["elo"] for r in elo_ratings.values()) / len(elo_ratings)
            else:
                avg_elo = DEFAULT_ELO
            elo_ratings[name] = {"elo": avg_elo, "games": 0}
            save_elo_ratings(elo_ratings)

    # Show starting Elo
    print(f"\nStarting Elo:")
    for name in engine_names:
        data = elo_ratings[name]
        prov = "?" if data["games"] < PROVISIONAL_GAMES else ""
        print(f"  {name}: {data['elo']:.0f} ({data['games']} games){prov}")
    print(flush=True)

    # Create pairings
    pairings = list(combinations(engine_names, 2))

    # Calculate total games: positions * 2 (both colors) * pairings
    total_games = len(positions) * 2 * len(pairings)
    print(f"Total games: {total_games} ({len(positions)} positions x 2 colors x {len(pairings)} pairings)")
    print()

    # Session tracking
    session_points = {name: 0.0 for name in engine_names}
    session_games = {name: 0 for name in engine_names}

    # Create PGN file
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    pgn_file = results_dir / f"epd_{epd_file.stem}_{timestamp}.pgn"
    with open(pgn_file, "w") as f:
        pass

    game_num = 0

    # Iterate through each position
    for pos_idx, (fen, pos_id) in enumerate(positions):
        print(f"\n--- Position {pos_idx + 1}/{len(positions)}: {pos_id} ---")

        # For each pairing, play the position twice (swap colors)
        for engine1_name, engine2_name in pairings:
            engine1 = engine_info[engine1_name]
            engine2 = engine_info[engine2_name]

            # Game 1: engine1 as white
            game_num += 1
            result, game = play_game(
                engine1["path"], engine2["path"],
                engine1_name, engine2_name,
                time_per_move, fen, pos_id,
                engine1["uci_options"], engine2["uci_options"]
            )

            # Save to PGN
            with open(pgn_file, "a") as f:
                print(game, file=f)
                print(file=f)

            # Note: We don't update global Elo in EPD mode since these are
            # specialized positions that would skew ratings

            # Update session stats
            session_games[engine1_name] += 1
            session_games[engine2_name] += 1
            if result == "1-0":
                session_points[engine1_name] += 1
            elif result == "0-1":
                session_points[engine2_name] += 1
            elif result == "1/2-1/2":
                session_points[engine1_name] += 0.5
                session_points[engine2_name] += 0.5

            print(f"  Game {game_num}/{total_games}: {engine1_name} vs {engine2_name} -> {result}")

            # Game 2: engine2 as white (swap colors)
            game_num += 1
            result, game = play_game(
                engine2["path"], engine1["path"],
                engine2_name, engine1_name,
                time_per_move, fen, pos_id,
                engine2["uci_options"], engine1["uci_options"]
            )

            # Save to PGN
            with open(pgn_file, "a") as f:
                print(game, file=f)
                print(file=f)

            # Update session stats
            session_games[engine1_name] += 1
            session_games[engine2_name] += 1
            if result == "1-0":
                session_points[engine2_name] += 1
            elif result == "0-1":
                session_points[engine1_name] += 1
            elif result == "1/2-1/2":
                session_points[engine1_name] += 0.5
                session_points[engine2_name] += 0.5

            print(f"  Game {game_num}/{total_games}: {engine2_name} vs {engine1_name} -> {result}")

        # Show standings after each position
        print_league_table(
            elo_ratings, set(engine_names), session_games,
            session_points, 0, False, True, game_num, total_games
        )

    # Final summary
    print(f"\n{'='*70}")
    print("EPD COMPETITION COMPLETE")
    print(f"{'='*70}")
    print(f"Positions: {len(positions)}")
    print(f"Games: {total_games}")
    print(f"PGN: {pgn_file}")

    print(f"\nFinal Standings:")
    sorted_results = sorted(session_points.items(), key=lambda x: -x[1])
    for rank, (name, points) in enumerate(sorted_results, 1):
        games = session_games[name]
        prov = "?" if elo_ratings[name]["games"] < PROVISIONAL_GAMES else ""
        print(f"  {rank}. {name}: {points:.1f}/{games} (Elo: {elo_ratings[name]['elo']:.0f}{prov})")

    print(f"{'='*70}")


def run_league(engine_names: list[str], engine_dir: Path,
               games_per_pairing: int, time_per_move: float, results_dir: Path,
               use_opening_book: bool = True):
    """Run a round-robin league with interleaved pairings."""

    pairings = list(combinations(engine_names, 2))
    num_pairings = len(pairings)

    # Games per pairing should be even (play each opening from both sides)
    if games_per_pairing % 2 != 0:
        games_per_pairing += 1
        print(f"Adjusted games per pairing to {games_per_pairing} (must be even)")

    # Number of complete rounds (each round = 2 games per pairing = one opening played both ways)
    num_rounds = games_per_pairing // 2
    total_games = num_rounds * num_pairings * 2

    # Load persistent Elo ratings
    elo_ratings = load_elo_ratings()

    # Initialize new engines with average Elo
    competitors = set(engine_names)
    for name in engine_names:
        if name not in elo_ratings:
            if elo_ratings:
                avg_elo = sum(r["elo"] for r in elo_ratings.values()) / len(elo_ratings)
            else:
                avg_elo = DEFAULT_ELO
            elo_ratings[name] = {"elo": avg_elo, "games": 0}
            save_elo_ratings(elo_ratings)

    # Track games and points in this competition
    games_this_comp = {name: 0 for name in engine_names}
    points_this_comp = {name: 0.0 for name in engine_names}

    print(f"\n{'='*95}")
    print("ROUND ROBIN TOURNAMENT")
    print(f"{'='*95}")
    print(f"Engines: {', '.join(engine_names)}")
    print(f"Pairings: {num_pairings}")
    print(f"Games per pairing: {games_per_pairing}")
    print(f"Rounds: {num_rounds} (each round = 2 games per pairing)")
    print(f"Total games: {total_games}")
    print(f"Time: {time_per_move}s/move")
    if use_opening_book:
        print(f"Opening book: {len(OPENING_BOOK)} positions")
    print(f"Elo ratings: {get_elo_file_path()}")
    print(f"{'='*95}")

    # Show starting Elo ratings
    print("\nStarting Elo ratings for competitors:")
    for name in sorted(engine_names, key=lambda n: -elo_ratings[n]["elo"]):
        data = elo_ratings[name]
        prov = "?" if data["games"] < PROVISIONAL_GAMES else ""
        print(f"  {name}: {data['elo']:.0f} ({data['games']} games){prov}")
    print()

    # Initialize head-to-head tracking (for summary at end)
    head_to_head = {}
    for e1, e2 in pairings:
        key = (e1, e2) if e1 < e2 else (e2, e1)
        head_to_head[key] = (0, 0, 0)

    # Prepare openings
    if use_opening_book:
        openings = OPENING_BOOK.copy()
        random.shuffle(openings)
        # Extend if we need more
        while len(openings) < num_rounds:
            extra = OPENING_BOOK.copy()
            random.shuffle(extra)
            openings.extend(extra)
    else:
        openings = [(None, None)] * num_rounds

    # Get engine paths and UCI options
    engine_info = {name: get_engine_info(name, engine_dir) for name in engine_names}

    # Create PGN file
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    pgn_file = results_dir / f"league_{timestamp}.pgn"
    with open(pgn_file, "w") as f:
        f.write(f"; Round Robin Tournament\n")
        f.write(f"; Engines: {', '.join(engine_names)}\n")
        f.write(f"; Date: {datetime.now().strftime('%Y-%m-%d %H:%M')}\n\n")

    game_num = 0

    # Play rounds
    for round_idx in range(num_rounds):
        opening_fen, opening_name = openings[round_idx]
        print(f"\n--- Round {round_idx + 1}/{num_rounds}: {opening_name or 'Starting position'} ---\n")

        # Play each pairing twice (once per color) for this opening
        for pairing_idx, (engine1, engine2) in enumerate(pairings):
            match_label = f"Match {pairing_idx + 1}/{num_pairings}"

            for color_swap in [False, True]:
                game_num += 1

                if color_swap:
                    white, black = engine2, engine1
                else:
                    white, black = engine1, engine2

                white_path, white_uci = engine_info[white]
                black_path, black_uci = engine_info[black]

                result, game = play_game(white_path, black_path, white, black,
                                         time_per_move, opening_fen, opening_name,
                                         white_uci, black_uci)

                # Append to PGN
                with open(pgn_file, "a") as f:
                    print(game, file=f)
                    print(file=f)

                # Update persistent Elo ratings
                update_elo_after_game(elo_ratings, white, black, result)

                # Update games and points for this competition
                games_this_comp[white] += 1
                games_this_comp[black] += 1

                if result == "1-0":
                    points_this_comp[white] += 1.0
                elif result == "0-1":
                    points_this_comp[black] += 1.0
                elif result == "1/2-1/2":
                    points_this_comp[white] += 0.5
                    points_this_comp[black] += 0.5

                # Update head-to-head tracking
                key = (engine1, engine2) if engine1 < engine2 else (engine2, engine1)
                e1_wins, e2_wins, draws = head_to_head[key]

                if result == "1-0":
                    if white == key[0]:
                        e1_wins += 1
                    else:
                        e2_wins += 1
                elif result == "0-1":
                    if black == key[0]:
                        e1_wins += 1
                    else:
                        e2_wins += 1
                elif result == "1/2-1/2":
                    draws += 1

                head_to_head[key] = (e1_wins, e2_wins, draws)

                # Print game result
                color_label = "(colors swapped)" if color_swap else ""
                print(f"Game {game_num:3d}/{total_games} {match_label}: {white} vs {black} -> {result} {color_label}")

                # Print compact standings after each game
                print_league_table(elo_ratings, competitors, games_this_comp, points_this_comp,
                                   round_idx + 1, competitors_only=True, game_num=game_num, total_games=total_games)

    # Print final standings (full table)
    print_league_table(elo_ratings, competitors, games_this_comp, points_this_comp, num_rounds, is_final=True)

    # Print head-to-head results
    print(f"{'='*95}")
    print("HEAD-TO-HEAD RESULTS (this competition)")
    print(f"{'='*95}")
    for (e1, e2), (w1, w2, d) in sorted(head_to_head.items()):
        total = w1 + w2 + d
        if total > 0:
            elo_diff, elo_err = calculate_elo_difference(w1, w2, d)
            print(f"{e1} vs {e2}: +{w1} -{w2} ={d}  (Elo diff: {elo_diff:+.0f} ±{elo_err:.0f})")
    print(f"{'='*95}")

    print(f"\nPGN saved to: {pgn_file}")
    print(f"Elo ratings saved to: {get_elo_file_path()}\n")


def run_gauntlet(challenger_name: str, engine_dir: Path,
                 num_rounds: int, time_per_move: float, results_dir: Path):
    """
    Test a challenger engine against all other engines in the engines directory.
    Plays in rounds: each round consists of 2 games (1 as white, 1 as black) against each opponent.
    Each game uses a random opening.
    """
    # Find all active engines except the challenger
    all_engines = get_active_engines(engine_dir)
    opponents = [e for e in all_engines if e != challenger_name]

    if not opponents:
        print(f"Error: No opponent engines found in {engine_dir}")
        sys.exit(1)

    games_per_opponent = num_rounds * 2  # 2 games per round (1 white, 1 black)
    total_games = len(opponents) * games_per_opponent

    # Load persistent Elo ratings
    elo_ratings = load_elo_ratings()

    # Initialize challenger if needed
    if challenger_name not in elo_ratings:
        if elo_ratings:
            avg_elo = sum(r["elo"] for r in elo_ratings.values()) / len(elo_ratings)
        else:
            avg_elo = DEFAULT_ELO
        elo_ratings[challenger_name] = {"elo": avg_elo, "games": 0}
        save_elo_ratings(elo_ratings)

    # Initialize opponents if needed
    for opponent in opponents:
        if opponent not in elo_ratings:
            if elo_ratings:
                avg_elo = sum(r["elo"] for r in elo_ratings.values()) / len(elo_ratings)
            else:
                avg_elo = DEFAULT_ELO
            elo_ratings[opponent] = {"elo": avg_elo, "games": 0}
            save_elo_ratings(elo_ratings)

    # Store starting Elo
    start_elo = elo_ratings[challenger_name]["elo"]

    print(f"\n{'='*70}")
    print("GAUNTLET TEST")
    print(f"{'='*70}")
    print(f"Challenger: {challenger_name}")
    print(f"Opponents: {len(opponents)}")
    print(f"Rounds: {num_rounds} (2 games per opponent per round)")
    print(f"Games per opponent: {games_per_opponent}")
    print(f"Total games: {total_games}")
    print(f"Time: {time_per_move}s/move")
    print(f"Opening book: {len(OPENING_BOOK)} positions (random selection)")
    print(f"Elo ratings: {get_elo_file_path()}")
    print(f"{'='*70}")

    # Show starting Elo
    print(f"\nChallenger starting Elo: {start_elo:.0f} ({elo_ratings[challenger_name]['games']} games)")
    print(f"\nOpponents:")
    for opp in sorted(opponents, key=lambda n: -elo_ratings.get(n, {}).get("elo", DEFAULT_ELO)):
        data = elo_ratings.get(opp, {"elo": DEFAULT_ELO, "games": 0})
        prov = "?" if data["games"] < PROVISIONAL_GAMES else ""
        print(f"  {opp}: {data['elo']:.0f} ({data['games']} games){prov}")
    print()

    # Get engine paths and UCI options
    challenger_path, challenger_uci = get_engine_info(challenger_name, engine_dir)
    opponent_info = {opp: get_engine_info(opp, engine_dir) for opp in opponents}

    # Create PGN file
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    pgn_file = results_dir / f"gauntlet_{challenger_name}_{timestamp}.pgn"
    with open(pgn_file, "w") as f:
        f.write(f"; Gauntlet Test\n")
        f.write(f"; Challenger: {challenger_name}\n")
        f.write(f"; Date: {datetime.now().strftime('%Y-%m-%d %H:%M')}\n\n")

    # Track results per opponent
    results_per_opponent = {opp: {"wins": 0, "losses": 0, "draws": 0} for opp in opponents}
    game_num = 0

    # Play in rounds
    for round_idx in range(num_rounds):
        print(f"\n--- Round {round_idx + 1}/{num_rounds} ---\n")

        for opponent in opponents:
            opponent_path, opponent_uci = opponent_info[opponent]

            # Game 1: Challenger as white
            game_num += 1
            opening_fen, opening_name = random.choice(OPENING_BOOK)

            result, game = play_game(challenger_path, opponent_path,
                                      challenger_name, opponent,
                                      time_per_move, opening_fen, opening_name,
                                      challenger_uci, opponent_uci)

            with open(pgn_file, "a") as f:
                print(game, file=f)
                print(file=f)

            update_elo_after_game(elo_ratings, challenger_name, opponent, result)

            if result == "1-0":
                results_per_opponent[opponent]["wins"] += 1
            elif result == "0-1":
                results_per_opponent[opponent]["losses"] += 1
            elif result == "1/2-1/2":
                results_per_opponent[opponent]["draws"] += 1

            print(f"Game {game_num:3d}/{total_games}: {challenger_name} (W) vs {opponent} -> {result}  [{opening_name}]")

            # Game 2: Challenger as black
            game_num += 1
            opening_fen, opening_name = random.choice(OPENING_BOOK)

            result, game = play_game(opponent_path, challenger_path,
                                      opponent, challenger_name,
                                      time_per_move, opening_fen, opening_name,
                                      opponent_uci, challenger_uci)

            with open(pgn_file, "a") as f:
                print(game, file=f)
                print(file=f)

            update_elo_after_game(elo_ratings, opponent, challenger_name, result)

            if result == "0-1":
                results_per_opponent[opponent]["wins"] += 1
            elif result == "1-0":
                results_per_opponent[opponent]["losses"] += 1
            elif result == "1/2-1/2":
                results_per_opponent[opponent]["draws"] += 1

            print(f"Game {game_num:3d}/{total_games}: {opponent} vs {challenger_name} (B) -> {result}  [{opening_name}]")

        # Show standings after each round
        print(f"\n--- Standings after round {round_idx + 1} ---")
        total_w = sum(r["wins"] for r in results_per_opponent.values())
        total_l = sum(r["losses"] for r in results_per_opponent.values())
        total_d = sum(r["draws"] for r in results_per_opponent.values())
        total_score = total_w + total_d * 0.5
        total_played = total_w + total_l + total_d
        current_elo = elo_ratings[challenger_name]["elo"]
        elo_change = current_elo - start_elo
        print(f"Overall: +{total_w} -{total_l} ={total_d}  Score: {total_score}/{total_played} ({total_score/total_played*100:.1f}%)  Elo: {current_elo:.0f} ({elo_change:+.0f})")

    # Print final summary
    print(f"\n{'='*70}")
    print("GAUNTLET RESULTS")
    print(f"{'='*70}")

    total_wins = sum(r["wins"] for r in results_per_opponent.values())
    total_losses = sum(r["losses"] for r in results_per_opponent.values())
    total_draws = sum(r["draws"] for r in results_per_opponent.values())
    total_score = total_wins + total_draws * 0.5
    total_played = total_wins + total_losses + total_draws

    print(f"\nChallenger: {challenger_name}")
    print(f"Overall: +{total_wins} -{total_losses} ={total_draws}  Score: {total_score}/{total_played} ({total_score/total_played*100:.1f}%)")

    overall_elo_diff, overall_elo_err = calculate_elo_difference(total_wins, total_losses, total_draws)
    print(f"Overall Elo performance: {overall_elo_diff:+.0f} ±{overall_elo_err:.0f}")

    print(f"\nResults per opponent:")
    print(f"{'Opponent':<30} {'W':>4} {'L':>4} {'D':>4} {'Score':>8} {'%':>7} {'Elo diff':>10}")
    print("-" * 70)

    for opponent in sorted(results_per_opponent.keys(), key=lambda o: elo_ratings.get(o, {}).get("elo", 0), reverse=True):
        r = results_per_opponent[opponent]
        w, l, d = r["wins"], r["losses"], r["draws"]
        score = w + d * 0.5
        total = w + l + d
        pct = score / total * 100 if total > 0 else 0
        elo_diff, _ = calculate_elo_difference(w, l, d)
        print(f"{opponent:<30} {w:>4} {l:>4} {d:>4} {score:>5.1f}/{total:<2} {pct:>6.1f}% {elo_diff:>+9.0f}")

    # Show Elo change
    final_elo = elo_ratings[challenger_name]["elo"]
    elo_change = final_elo - start_elo
    final_games = elo_ratings[challenger_name]["games"]
    prov = "?" if final_games < PROVISIONAL_GAMES else ""

    print(f"\nChallenger Elo: {start_elo:.0f} -> {final_elo:.0f} ({elo_change:+.0f}) - {final_games} games{prov}")
    print(f"\nPGN saved to: {pgn_file}")
    print(f"Elo ratings saved to: {get_elo_file_path()}")
    print(f"{'='*70}\n")


def run_random(engine_dir: Path, num_matches: int, time_per_move: float, results_dir: Path, weighted: bool = False,
               time_low: float = None, time_high: float = None):
    """
    Randomly select pairs of engines and play 2-game matches (1 white, 1 black).
    Each game uses a random opening.

    If weighted=True, the first engine is selected with bias toward fewer games
    (weight = 1 / (games + 1)), while the second engine is chosen purely at random.
    This ensures under-tested engines get priority but play diverse opponents
    for better Elo calibration.

    If time_low and time_high are provided, randomly select a time for each match
    from that range. Otherwise use time_per_move.
    """
    use_time_range = time_low is not None and time_high is not None
    # Find all active engines
    all_engines = get_active_engines(engine_dir)

    if len(all_engines) < 2:
        print(f"Error: Need at least 2 engines in {engine_dir}, found {len(all_engines)}")
        sys.exit(1)

    total_games = num_matches * 2

    # Load persistent Elo ratings
    elo_ratings = load_elo_ratings()

    # Initialize any missing engines
    for engine in all_engines:
        if engine not in elo_ratings:
            if elo_ratings:
                avg_elo = sum(r["elo"] for r in elo_ratings.values()) / len(elo_ratings)
            else:
                avg_elo = DEFAULT_ELO
            elo_ratings[engine] = {"elo": avg_elo, "games": 0}
            save_elo_ratings(elo_ratings)

    print(f"\n{'='*70}")
    print(f"RANDOM MODE{' (WEIGHTED)' if weighted else ''}")
    print(f"{'='*70}")
    print(f"Active engines: {len(all_engines)}")
    if weighted:
        print("Selection: first engine weighted (fewer games = higher chance), second random")
    print(f"Matches: {num_matches} (2 games each = {total_games} total games)")
    if use_time_range:
        print(f"Time: {time_low}-{time_high}s/move (random per match)")
    else:
        print(f"Time: {time_per_move}s/move")
    print(f"Opening book: {len(OPENING_BOOK)} positions (random selection)")
    print(f"Elo ratings: {get_elo_file_path()}")
    print(f"{'='*70}")

    # Create PGN file
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    pgn_file = results_dir / f"random_{timestamp}.pgn"
    with open(pgn_file, "w") as f:
        f.write(f"; Random Mode\n")
        f.write(f"; Date: {datetime.now().strftime('%Y-%m-%d %H:%M')}\n\n")

    game_num = 0

    # Track session stats for standings display
    session_engines = set()
    session_games = {}  # engine -> games played this session
    session_points = {}  # engine -> points this session

    for match_idx in range(num_matches):
        # Pick two engines (weighted or uniform random)
        if weighted:
            # Weight = 1 / (games + 1) - fewer games means higher weight
            weights = [1.0 / (elo_ratings[e]["games"] + 1) for e in all_engines]
            engine1 = random.choices(all_engines, weights=weights, k=1)[0]
            # Pick engine2 purely at random (not weighted) for better Elo calibration
            remaining = [e for e in all_engines if e != engine1]
            engine2 = random.choice(remaining)
        else:
            engine1, engine2 = random.sample(all_engines, 2)
        engine1_path, engine1_uci = get_engine_info(engine1, engine_dir)
        engine2_path, engine2_uci = get_engine_info(engine2, engine_dir)

        # Select time for this match (random from range, or fixed)
        if use_time_range:
            match_time = random.uniform(time_low, time_high)
        else:
            match_time = time_per_move

        print(f"\n--- Match {match_idx + 1}/{num_matches}: {engine1} vs {engine2} ({match_time:.2f}s/move) ---\n")

        # Game 1: engine1 as white
        game_num += 1
        opening_fen, opening_name = random.choice(OPENING_BOOK)

        result, game = play_game(engine1_path, engine2_path,
                                  engine1, engine2,
                                  match_time, opening_fen, opening_name,
                                  engine1_uci, engine2_uci)

        with open(pgn_file, "a") as f:
            print(game, file=f)
            print(file=f)

        update_elo_after_game(elo_ratings, engine1, engine2, result)

        # Track session stats
        for eng in [engine1, engine2]:
            session_engines.add(eng)
            session_games[eng] = session_games.get(eng, 0) + 1
            if eng not in session_points:
                session_points[eng] = 0.0
        if result == "1-0":
            session_points[engine1] += 1.0
        elif result == "0-1":
            session_points[engine2] += 1.0
        elif result == "1/2-1/2":
            session_points[engine1] += 0.5
            session_points[engine2] += 0.5

        print(f"Game {game_num:3d}/{total_games}: {engine1} (W) vs {engine2} -> {result}  [{opening_name}]")
        print_league_table(elo_ratings, session_engines, session_games, session_points,
                           0, competitors_only=True, game_num=game_num, total_games=total_games)

        # Game 2: engine2 as white (same time control as game 1)
        game_num += 1
        opening_fen, opening_name = random.choice(OPENING_BOOK)

        result, game = play_game(engine2_path, engine1_path,
                                  engine2, engine1,
                                  match_time, opening_fen, opening_name,
                                  engine2_uci, engine1_uci)

        with open(pgn_file, "a") as f:
            print(game, file=f)
            print(file=f)

        update_elo_after_game(elo_ratings, engine2, engine1, result)

        # Track session stats
        for eng in [engine1, engine2]:
            session_games[eng] = session_games.get(eng, 0) + 1
        if result == "1-0":
            session_points[engine2] += 1.0
        elif result == "0-1":
            session_points[engine1] += 1.0
        elif result == "1/2-1/2":
            session_points[engine1] += 0.5
            session_points[engine2] += 0.5

        print(f"Game {game_num:3d}/{total_games}: {engine2} (W) vs {engine1} -> {result}  [{opening_name}]")
        print_league_table(elo_ratings, session_engines, session_games, session_points,
                           0, competitors_only=True, game_num=game_num, total_games=total_games)

    # Print final standings
    print(f"\n{'='*70}")
    print("CURRENT ELO STANDINGS")
    print(f"{'='*70}")
    print(f"{'Rank':<6}{'Engine':<30}{'Elo':>8}{'Games':>8}")
    print("-" * 52)

    sorted_engines = sorted(elo_ratings.items(), key=lambda x: -x[1]["elo"])
    for rank, (name, data) in enumerate(sorted_engines, 1):
        prov = "?" if data["games"] < PROVISIONAL_GAMES else ""
        print(f"{rank:<6}{name:<30}{data['elo']:>8.0f}{data['games']:>7}{prov}")

    print(f"\nPGN saved to: {pgn_file}")
    print(f"Elo ratings saved to: {get_elo_file_path()}")
    print(f"{'='*70}\n")


def main():
    parser = argparse.ArgumentParser(
        description="Chess engine competition harness",
        epilog="Engine names can be shorthand (v1, v10) or full (v1-baseline, v10-arrayvec-movelist)"
    )
    parser.add_argument("engines", nargs="*",
                        help="Engine version names (e.g., v1, v10, or v1-baseline)")
    parser.add_argument("--games", "-g", type=int, default=100,
                        help="Number of games per pairing (default: 100)")
    parser.add_argument("--time", "-t", type=float, default=None,
                        help="Time per move in seconds (default: 1.0)")
    parser.add_argument("--timelow", type=float, default=None,
                        help="Minimum time per move (use with --timehigh for random range)")
    parser.add_argument("--timehigh", type=float, default=None,
                        help="Maximum time per move (use with --timelow for random range)")
    parser.add_argument("--no-book", action="store_true",
                        help="Disable opening book (start all games from initial position)")
    parser.add_argument("--gauntlet", action="store_true",
                        help="Gauntlet mode: test one engine against all others in engines directory")
    parser.add_argument("--random", action="store_true",
                        help="Random mode: randomly pair engines for 2-game matches")
    parser.add_argument("--weighted", action="store_true",
                        help="With --random: weight selection by inverse game count (fewer games = higher chance)")
    parser.add_argument("--epd", type=str, default=None,
                        help="EPD file mode: play through positions from an EPD file sequentially")

    args = parser.parse_args()

    # Validate time arguments
    if args.timelow is not None or args.timehigh is not None:
        if args.timelow is None or args.timehigh is None:
            print("Error: --timelow and --timehigh must be used together")
            sys.exit(1)
        if args.timelow > args.timehigh:
            print("Error: --timelow must be less than or equal to --timehigh")
            sys.exit(1)
        if args.time is not None:
            print("Error: Cannot use --time with --timelow/--timehigh")
            sys.exit(1)
        time_per_move = None  # Will use range
        time_low = args.timelow
        time_high = args.timehigh
    else:
        time_per_move = args.time if args.time is not None else 1.0
        time_low = None
        time_high = None

    script_dir = Path(__file__).parent
    engine_dir = script_dir.parent / "engines"
    results_dir = script_dir.parent / "results" / "competitions"
    results_dir.mkdir(parents=True, exist_ok=True)

    use_opening_book = not args.no_book

    # Resolve shorthand engine names to full names
    resolved_engines = [resolve_engine_name(e, engine_dir) for e in args.engines]

    # Print resolved names if different from input
    for orig, resolved in zip(args.engines, resolved_engines):
        if orig != resolved:
            print(f"Resolved '{orig}' -> '{resolved}'")

    if args.epd:
        # EPD mode: play through positions from an EPD file
        epd_path = Path(args.epd)
        if not epd_path.exists():
            # Try looking in engines/epd directory
            epd_path = engine_dir / "epd" / args.epd
            if not epd_path.exists():
                print(f"Error: EPD file not found: {args.epd}")
                print(f"Searched: {Path(args.epd).absolute()} and {epd_path}")
                sys.exit(1)
        if len(resolved_engines) < 2:
            print("Error: EPD mode requires at least 2 engines")
            sys.exit(1)
        if time_per_move is None:
            print("Error: --timelow/--timehigh not supported in EPD mode, use --time")
            sys.exit(1)
        run_epd(resolved_engines, engine_dir, epd_path, time_per_move, results_dir)
    elif args.random:
        # Random mode: randomly pair engines for matches
        if args.engines:
            print("Warning: Engine arguments ignored in random mode")
        run_random(engine_dir, args.games, time_per_move, results_dir, args.weighted, time_low, time_high)
    elif args.gauntlet:
        # Gauntlet mode: test one engine against all others
        if len(resolved_engines) != 1:
            print("Error: Gauntlet mode requires exactly one engine (the challenger)")
            sys.exit(1)
        if time_per_move is None:
            print("Error: --timelow/--timehigh not supported in gauntlet mode, use --time")
            sys.exit(1)
        run_gauntlet(resolved_engines[0], engine_dir, args.games, time_per_move, results_dir)
    elif len(resolved_engines) >= 3:
        # Round-robin league for 3+ engines
        if time_per_move is None:
            print("Error: --timelow/--timehigh not supported in league mode, use --time")
            sys.exit(1)
        run_league(resolved_engines, engine_dir, args.games, time_per_move, results_dir, use_opening_book)
    elif len(resolved_engines) == 2:
        # Head-to-head match for exactly 2 engines
        if time_per_move is None:
            print("Error: --timelow/--timehigh not supported in head-to-head mode, use --time")
            sys.exit(1)
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        pgn_file = results_dir / f"{resolved_engines[0]}_vs_{resolved_engines[1]}_{timestamp}.pgn"
        run_match(resolved_engines[0], resolved_engines[1], engine_dir,
                  args.games, time_per_move, pgn_file, use_opening_book)
    else:
        print("Error: At least 2 engines are required (or use --random or --gauntlet mode)")
        sys.exit(1)


if __name__ == "__main__":
    main()

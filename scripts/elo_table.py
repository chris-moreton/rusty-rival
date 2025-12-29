#!/usr/bin/env python3
"""Display ELO ratings in a live-updating table."""

import json
import os
import sys
import time
from pathlib import Path

RESULTS_FILE = Path(__file__).parent.parent / "results" / "elo_ratings.json"

def clear_screen():
    """Clear screen and move cursor to top."""
    print("\033[2J\033[H", end="")

def load_ratings():
    """Load ratings from JSON file."""
    if not RESULTS_FILE.exists():
        return {}
    with open(RESULTS_FILE) as f:
        return json.load(f)

def display_table(ratings):
    """Display ratings as a formatted table."""
    clear_screen()

    if not ratings:
        print("No ratings found.")
        print(f"\nWatching: {RESULTS_FILE}")
        return

    # Sort by ELO descending
    sorted_ratings = sorted(
        ratings.items(),
        key=lambda x: x[1]["elo"],
        reverse=True
    )

    # Calculate column widths
    name_width = max(len(name) for name in ratings.keys())
    name_width = max(name_width, 6)  # minimum "Engine"

    # Print header
    print(f"{'Rank':<5} {'Engine':<{name_width}}  {'ELO':>8}  {'Games':>6}")
    print("-" * (5 + name_width + 2 + 8 + 2 + 6))

    # Print rows
    for i, (name, data) in enumerate(sorted_ratings, 1):
        elo = data["elo"]
        games = data["games"]
        print(f"{i:<5} {name:<{name_width}}  {elo:>8.1f}  {games:>6}")

    print()
    print(f"Watching: {RESULTS_FILE}")
    print("Press Ctrl+C to exit")

def main():
    """Main loop - watch file and update display."""
    last_mtime = 0

    try:
        while True:
            try:
                current_mtime = RESULTS_FILE.stat().st_mtime if RESULTS_FILE.exists() else 0
            except OSError:
                current_mtime = 0

            if current_mtime != last_mtime:
                ratings = load_ratings()
                display_table(ratings)
                last_mtime = current_mtime

            time.sleep(0.5)
    except KeyboardInterrupt:
        print("\nExiting.")
        sys.exit(0)

if __name__ == "__main__":
    main()

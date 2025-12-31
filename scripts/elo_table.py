#!/usr/bin/env python3
"""Display ELO ratings in a live-updating table."""

import json
import re
import sys
import time
from pathlib import Path

# ANSI color codes - rainbow colors (ROYGBIV)
RAINBOW = [
    "\033[38;5;196m",  # Red (most recent)
    "\033[38;5;208m",  # Orange
    "\033[38;5;226m",  # Yellow
    "\033[38;5;46m",   # Green
    "\033[38;5;21m",   # Blue
    "\033[38;5;93m",   # Indigo
    "\033[38;5;129m",  # Violet (7th most recent)
]
BASELINE_COLOR = "\033[38;5;250m"  # Light gray for baseline
RESET = "\033[0m"

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

def find_recent_versions(ratings, count=7):
    """Find the N most recent versions of our engine (highest v### numbers).

    Returns a dict mapping version name to its recency rank (0 = most recent).
    """
    versions = []
    for name in ratings.keys():
        match = re.match(r'^v(\d+)', name)
        if match:
            num = int(match.group(1))
            versions.append((num, name))

    # Sort by version number descending, take top N
    versions.sort(reverse=True)
    recent = versions[:count]

    # Return dict mapping name -> rank (0 = most recent)
    return {name: rank for rank, (_, name) in enumerate(recent)}

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

    # Find recent versions to highlight with rainbow colors
    recent_versions = find_recent_versions(ratings, count=7)

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
        row = f"{i:<5} {name:<{name_width}}  {elo:>8.1f}  {games:>6}"
        if name in recent_versions:
            rank = recent_versions[name]
            print(f"{RAINBOW[rank]}{row}{RESET}")
        elif name == "v001-baseline":
            print(f"{BASELINE_COLOR}{row}{RESET}")
        else:
            print(row)

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

#!/usr/bin/env python3
"""
Migrate existing PGN files and engine data to PostgreSQL database.

Usage:
    python scripts/migrate_pgn.py

This script will:
1. Create database tables if they don't exist
2. Import engines from engines.json
3. Import Elo ratings from elo_ratings.json (if exists)
4. Parse all PGN files and import game results
"""

import json
import sys
from pathlib import Path
from datetime import datetime

import chess.pgn

# Add parent directory to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent))

from web.app import create_app
from web.database import db
from web.models import Engine, Game, EloRating


def parse_date(date_str: str):
    """Parse PGN date format YYYY.MM.DD."""
    if not date_str or date_str == '????.??.??':
        return datetime.now().date()
    try:
        return datetime.strptime(date_str, "%Y.%m.%d").date()
    except ValueError:
        return datetime.now().date()


def result_to_scores(result: str) -> tuple[float, float]:
    """Convert result string to (white_score, black_score)."""
    if result == "1-0":
        return 1.0, 0.0
    elif result == "0-1":
        return 0.0, 1.0
    elif result == "1/2-1/2":
        return 0.5, 0.5
    else:
        return 0.0, 0.0  # Incomplete game


def migrate_engines(project_root: Path):
    """Load engines from engines.json into database."""
    engines_file = project_root / "engines" / "engines.json"

    if not engines_file.exists():
        print(f"Warning: {engines_file} not found")
        return 0

    with open(engines_file) as f:
        engines_config = json.load(f)

    count = 0
    for name, config in engines_config.items():
        existing = Engine.query.filter_by(name=name).first()
        if not existing:
            engine = Engine(
                name=name,
                binary_path=config.get('binary'),
                active=config.get('active', True),
                initial_elo=config.get('initial_elo', 1500),
                uci_options=config.get('uci_options')
            )
            db.session.add(engine)
            count += 1

    db.session.commit()
    print(f"Imported {count} new engines (total: {len(engines_config)})")
    return count


def migrate_elo_ratings(project_root: Path):
    """Load Elo ratings from elo_ratings.json if it exists."""
    ratings_file = project_root / "results" / "elo_ratings.json"

    if not ratings_file.exists():
        print(f"Note: {ratings_file} not found, skipping Elo import")
        return 0

    with open(ratings_file) as f:
        ratings = json.load(f)

    count = 0
    for name, data in ratings.items():
        engine = Engine.query.filter_by(name=name).first()
        if not engine:
            # Create engine entry if it doesn't exist
            engine = Engine(name=name, active=False)
            db.session.add(engine)
            db.session.flush()  # Get the ID

        existing = EloRating.query.filter_by(engine_id=engine.id).first()
        if existing:
            existing.elo = data['elo']
            existing.games_played = data['games']
        else:
            rating = EloRating(
                engine_id=engine.id,
                elo=data['elo'],
                games_played=data['games']
            )
            db.session.add(rating)
            count += 1

    db.session.commit()
    print(f"Imported {count} new Elo ratings (total: {len(ratings)})")
    return count


def ensure_elo_ratings():
    """Ensure all engines have an Elo rating entry."""
    engines_without_rating = db.session.query(Engine).filter(
        ~Engine.id.in_(
            db.session.query(EloRating.engine_id)
        )
    ).all()

    for engine in engines_without_rating:
        rating = EloRating(
            engine_id=engine.id,
            elo=engine.initial_elo or 1500,
            games_played=0
        )
        db.session.add(rating)

    db.session.commit()
    if engines_without_rating:
        print(f"Created Elo ratings for {len(engines_without_rating)} engines without ratings")


def migrate_pgn_files(project_root: Path):
    """Parse all PGN files and insert games into database."""
    pgn_dir = project_root / "results" / "competitions"

    if not pgn_dir.exists():
        print(f"Warning: {pgn_dir} not found")
        return 0

    pgn_files = list(pgn_dir.glob("*.pgn"))
    print(f"Found {len(pgn_files)} PGN files to process")

    # Build engine name -> id lookup
    engines = {e.name: e.id for e in Engine.query.all()}

    total_games = 0
    skipped_unknown_engine = 0
    skipped_incomplete = 0

    for pgn_file in sorted(pgn_files):
        file_games = 0
        with open(pgn_file) as f:
            while True:
                try:
                    game = chess.pgn.read_game(f)
                except Exception as e:
                    print(f"  Error parsing game in {pgn_file.name}: {e}")
                    continue

                if game is None:
                    break

                white = game.headers.get("White")
                black = game.headers.get("Black")
                result = game.headers.get("Result", "*")

                # Skip incomplete games
                if result == "*":
                    skipped_incomplete += 1
                    continue

                # Skip if engines not in database
                if white not in engines:
                    skipped_unknown_engine += 1
                    continue
                if black not in engines:
                    skipped_unknown_engine += 1
                    continue

                white_score, black_score = result_to_scores(result)

                game_record = Game(
                    white_engine_id=engines[white],
                    black_engine_id=engines[black],
                    result=result,
                    white_score=white_score,
                    black_score=black_score,
                    date_played=parse_date(game.headers.get("Date", "")),
                    time_control=game.headers.get("TimeControl"),
                    opening_name=game.headers.get("Opening"),
                    opening_fen=game.headers.get("FEN"),
                    pgn_file=pgn_file.name
                )
                db.session.add(game_record)
                file_games += 1
                total_games += 1

        # Commit per file to avoid memory issues
        db.session.commit()
        if file_games > 0:
            print(f"  {pgn_file.name}: {file_games} games")

    print(f"\nImported {total_games} games")
    if skipped_unknown_engine > 0:
        print(f"  Skipped {skipped_unknown_engine} games (unknown engine)")
    if skipped_incomplete > 0:
        print(f"  Skipped {skipped_incomplete} games (incomplete)")

    return total_games


def main():
    """Run the migration."""
    project_root = Path(__file__).parent.parent

    print("=" * 60)
    print("Chess Engine Competition Database Migration")
    print("=" * 60)

    app = create_app()

    with app.app_context():
        # Check database connection
        try:
            db.engine.connect()
            print(f"\nConnected to database")
        except Exception as e:
            print(f"\nError connecting to database: {e}")
            print("Make sure DATABASE_URL is set in .env file")
            sys.exit(1)

        # Create tables
        print("\nCreating database tables...")
        db.create_all()
        print("Tables created")

        # Run migrations
        print("\n--- Importing Engines ---")
        migrate_engines(project_root)

        print("\n--- Importing Elo Ratings ---")
        migrate_elo_ratings(project_root)
        ensure_elo_ratings()

        print("\n--- Importing Games from PGN Files ---")
        migrate_pgn_files(project_root)

        # Print summary
        print("\n" + "=" * 60)
        print("Migration Complete!")
        print("=" * 60)
        print(f"  Engines: {Engine.query.count()}")
        print(f"  Elo Ratings: {EloRating.query.count()}")
        print(f"  Games: {Game.query.count()}")


if __name__ == "__main__":
    main()

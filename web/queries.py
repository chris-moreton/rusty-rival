"""
Database queries for the competition dashboard.
"""

from sqlalchemy import func


def get_db():
    """Get db instance (late import to avoid circular imports)."""
    from web.database import db
    return db


def get_models():
    """Get models (late import to avoid circular imports)."""
    from web.models import Engine, Game, EloRating
    return Engine, Game, EloRating


def get_engines_ranked_by_elo(active_only=True):
    """
    Get engines sorted by Elo rating descending.
    Returns list of (Engine, EloRating) tuples.
    """
    db = get_db()
    Engine, Game, EloRating = get_models()

    query = db.session.query(Engine, EloRating).join(
        EloRating, Engine.id == EloRating.engine_id
    )

    if active_only:
        query = query.filter(Engine.active == True)

    return query.order_by(EloRating.elo.desc()).all()


def get_h2h_raw_data():
    """
    Get raw head-to-head data from games table.
    Returns dict: {(white_id, black_id): {'white_points': float, 'black_points': float, 'games': int}}
    """
    db = get_db()
    Engine, Game, EloRating = get_models()

    results = db.session.query(
        Game.white_engine_id,
        Game.black_engine_id,
        func.sum(Game.white_score).label('white_points'),
        func.sum(Game.black_score).label('black_points'),
        func.count(Game.id).label('total_games')
    ).group_by(
        Game.white_engine_id,
        Game.black_engine_id
    ).all()

    h2h = {}
    for row in results:
        key = (row.white_engine_id, row.black_engine_id)
        h2h[key] = {
            'white_points': float(row.white_points or 0),
            'black_points': float(row.black_points or 0),
            'games': row.total_games
        }

    return h2h


def build_h2h_grid(engines, h2h_raw):
    """
    Build the H2H grid for the dashboard.

    Args:
        engines: List of (Engine, EloRating) tuples, sorted by Elo
        h2h_raw: Raw H2H data from get_h2h_raw_data()

    Returns:
        List of row dicts with engine info and cells
    """
    # Build lookups
    engine_elos = {e.Engine.id: float(e.EloRating.elo) for e in engines}

    grid = []
    for row_idx, row_engine in enumerate(engines):
        row_id = row_engine.Engine.id
        row_elo = engine_elos[row_id]
        row_rank = row_idx + 1

        cells = []
        for col_idx, col_engine in enumerate(engines):
            col_id = col_engine.Engine.id
            col_elo = engine_elos[col_id]
            col_rank = col_idx + 1

            # Same engine - diagonal
            if row_id == col_id:
                cells.append({
                    'score': '-',
                    'color': 'diagonal',
                    'games': 0,
                    'tooltip': ''
                })
                continue

            # Get H2H data from both directions
            # row_engine as white vs col_engine
            as_white = h2h_raw.get((row_id, col_id), {'white_points': 0, 'black_points': 0, 'games': 0})
            # row_engine as black vs col_engine (col as white)
            as_black = h2h_raw.get((col_id, row_id), {'white_points': 0, 'black_points': 0, 'games': 0})

            # Calculate row_engine's total points against col_engine
            row_points = as_white['white_points'] + as_black['black_points']
            col_points = as_white['black_points'] + as_black['white_points']
            total_games = as_white['games'] + as_black['games']

            if total_games == 0:
                cells.append({
                    'score': '-',
                    'color': 'no-games',
                    'games': 0,
                    'tooltip': 'No games played'
                })
                continue

            # Determine color based on Elo ranking vs actual result
            # row_rank < col_rank means row_engine is higher rated (ranked higher = lower number)
            row_is_higher_rated = row_rank < col_rank

            # Calculate if row_engine is winning, losing, or drawing
            if row_points > col_points:
                row_winning = True
                row_drawing = False
            elif row_points < col_points:
                row_winning = False
                row_drawing = False
            else:
                row_winning = False
                row_drawing = True

            # Color logic:
            # RED: Higher-rated engine losing OR drawing H2H
            # GREEN: Lower-rated engine winning OR drawing H2H
            # NEUTRAL: Expected results (higher winning, lower losing)
            if row_is_higher_rated:
                # Row is higher rated
                if not row_winning:  # Losing or drawing
                    color = 'red'
                else:
                    color = 'neutral'
            else:
                # Row is lower rated
                if row_winning or row_drawing:  # Winning or drawing
                    color = 'green'
                else:
                    color = 'neutral'

            cells.append({
                'score': f"{row_points:.0f}-{col_points:.0f}",
                'color': color,
                'games': total_games,
                'tooltip': f"{total_games} games"
            })

        grid.append({
            'rank': row_rank,
            'engine_name': row_engine.Engine.name,
            'elo': row_elo,
            'games_played': row_engine.EloRating.games_played,
            'cells': cells
        })

    return grid


def get_dashboard_data(active_only=True):
    """
    Get all data needed for the dashboard.
    Returns (engines, grid, column_headers).
    """
    engines = get_engines_ranked_by_elo(active_only=active_only)

    if not engines:
        return [], [], []

    h2h_raw = get_h2h_raw_data()
    grid = build_h2h_grid(engines, h2h_raw)
    column_headers = [(i + 1, e.Engine.name) for i, e in enumerate(engines)]

    return engines, grid, column_headers

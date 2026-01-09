"""
Database queries for the competition dashboard.
"""

import math
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


def calculate_expected_score(elo_a: float, elo_b: float, num_games: int) -> float:
    """
    Calculate expected score for player A against player B over num_games.
    Uses standard Elo formula: E = 1 / (1 + 10^((Rb - Ra) / 400))
    """
    if num_games == 0:
        return 0
    expected_per_game = 1 / (1 + 10 ** ((elo_b - elo_a) / 400))
    return expected_per_game * num_games


def deviation_to_color(deviation: float, is_overperforming: bool, num_games: int) -> str:
    """
    Convert deviation from expected to a background color.

    Args:
        deviation: How many points above/below expected (absolute value)
        is_overperforming: True for green (lower-rated winning), False for red (higher-rated losing)
        num_games: Number of games played (affects scaling)

    Returns:
        CSS color string (rgba or hex)
    """
    if num_games == 0:
        return '#f5f5f5'  # No games - neutral gray

    # Normalize deviation by number of games to get a percentage
    # A deviation of 10% of games is significant
    deviation_pct = abs(deviation) / num_games if num_games > 0 else 0

    # Scale intensity: 0% deviation = very light, 50%+ deviation = full color
    # Using a curve to make small deviations more visible
    intensity = min(1.0, deviation_pct * 2)  # 50% deviation = full intensity

    # Apply sqrt to make small deviations more visible
    intensity = math.sqrt(intensity)

    if is_overperforming:
        # Green: from very light (#e8f5e9) to darker (#4caf50)
        # RGB interpolation
        r = int(232 - (232 - 76) * intensity)   # 232 -> 76
        g = int(245 - (245 - 175) * intensity)  # 245 -> 175
        b = int(233 - (233 - 80) * intensity)   # 233 -> 80
    else:
        # Red: from very light (#ffebee) to darker (#e57373)
        r = int(255 - (255 - 229) * intensity)  # 255 -> 229
        g = int(235 - (235 - 115) * intensity)  # 235 -> 115
        b = int(238 - (238 - 115) * intensity)  # 238 -> 115

    return f'rgb({r}, {g}, {b})'


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
                    'bg_color': '#e0e0e0',
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
                    'bg_color': '#f5f5f5',
                    'games': 0,
                    'tooltip': 'No games played'
                })
                continue

            # Calculate expected score for row_engine against col_engine
            expected_row = calculate_expected_score(row_elo, col_elo, total_games)
            deviation = row_points - expected_row  # Positive = overperforming, negative = underperforming

            # Determine if row_engine is higher or lower rated
            row_is_higher_rated = row_rank < col_rank

            # Color logic:
            # - Higher-rated engine underperforming (deviation < 0) -> RED
            # - Lower-rated engine overperforming (deviation > 0) -> GREEN
            # - Results match expectations -> neutral/white

            if row_is_higher_rated and deviation < 0:
                # Higher-rated losing more than expected -> RED
                bg_color = deviation_to_color(deviation, is_overperforming=False, num_games=total_games)
            elif not row_is_higher_rated and deviation > 0:
                # Lower-rated winning more than expected -> GREEN
                bg_color = deviation_to_color(deviation, is_overperforming=True, num_games=total_games)
            else:
                # Results roughly as expected or better than expected for higher-rated
                bg_color = '#f9f9f9'  # Neutral

            # Build tooltip with more detail
            expected_str = f"{expected_row:.1f}"
            if deviation > 0:
                dev_str = f"+{deviation:.1f}"
            else:
                dev_str = f"{deviation:.1f}"
            tooltip = f"{total_games} games | Expected: {expected_str} | Actual: {row_points:.0f} ({dev_str})"

            cells.append({
                'score': f"{row_points:.0f}-{col_points:.0f}",
                'bg_color': bg_color,
                'games': total_games,
                'tooltip': tooltip
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

"""
Database queries for the competition dashboard.
"""

import math
import re
from sqlalchemy import func


def get_engine_type(name: str) -> str:
    """
    Determine the engine type from its name.

    Returns:
        'stockfish' - for sf-* engines
        'official' - for vX.X.X official releases (e.g., v1.0.13)
        'dev' - for development versions (e.g., v031-delta-pruning)
        'other' - for everything else
    """
    if name.startswith('sf-'):
        return 'stockfish'
    # Official releases: vX.X.X pattern (semantic versioning)
    if re.match(r'^v\d+\.\d+\.\d+$', name):
        return 'official'
    # Dev versions: vXXX-name pattern
    if re.match(r'^v\d{2,3}-', name):
        return 'dev'
    return 'other'


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


def stability_to_color(stability: int) -> str:
    """
    Convert stability score (0-100) to a background color.

    - 100 = dark green (perfect match with expected)
    - 50 = white/neutral
    - 0 = dark red (poor match)

    Args:
        stability: Score from 0-100

    Returns:
        CSS color string
    """
    if stability >= 50:
        # Green gradient: 50 = white, 100 = dark green (#4caf50)
        intensity = (stability - 50) / 50.0  # 0 to 1
        r = int(255 - (255 - 76) * intensity)   # 255 -> 76
        g = int(255 - (255 - 175) * intensity)  # 255 -> 175
        b = int(255 - (255 - 80) * intensity)   # 255 -> 80
    else:
        # Red gradient: 50 = white, 0 = dark red (#e57373)
        intensity = (50 - stability) / 50.0  # 0 to 1
        r = int(255 - (255 - 229) * intensity)  # 255 -> 229
        g = int(255 - (255 - 115) * intensity)  # 255 -> 115
        b = int(255 - (255 - 115) * intensity)  # 255 -> 115

    return f'rgb({r}, {g}, {b})'


def calculate_excess_deviation(deviation: float, num_games: int) -> float:
    """
    Calculate how many standard deviations beyond tolerance the result is.
    Returns 0 if within tolerance, positive value if outside.

    Args:
        deviation: Points above/below expected
        num_games: Number of games played

    Returns:
        Excess deviation in standard deviation units (0 = within tolerance)
    """
    if num_games == 0:
        return 0.0

    # Same tolerance calculation as deviation_to_color
    tolerance = 0.5 * math.sqrt(num_games)
    tolerance = max(1.0, tolerance)

    excess = abs(deviation) - tolerance
    if excess <= 0:
        return 0.0

    # Normalize by tolerance to get "excess standard deviations"
    return excess / tolerance


def deviation_to_color(deviation: float, num_games: int) -> str:
    """
    Convert deviation from expected to a background color.

    - White/neutral if within tolerance of expected (normal variance)
    - Green if overperforming (positive deviation beyond tolerance)
    - Red if underperforming (negative deviation beyond tolerance)

    Args:
        deviation: Points above (positive) or below (negative) expected
        num_games: Number of games played

    Returns:
        CSS color string
    """
    if num_games == 0:
        return '#f5f5f5'  # No games - neutral gray

    # Tolerance based on statistical variance
    # Standard deviation of game outcomes ≈ 0.5 * sqrt(n)
    # Use ~1 standard deviation as tolerance (generous)
    tolerance = 0.5 * math.sqrt(num_games)
    # Minimum tolerance of 1.0 points for very small sample sizes
    tolerance = max(1.0, tolerance)

    # If within tolerance, return neutral white
    if abs(deviation) <= tolerance:
        return '#ffffff'

    # Calculate excess deviation beyond tolerance
    excess = abs(deviation) - tolerance

    # Scale intensity: how much beyond tolerance relative to expected variance
    # Use 2 more standard deviations as the range for full intensity
    # So total range is 1 SD (tolerance) to 3 SD (full color)
    full_intensity_threshold = tolerance * 2  # 2 more SDs
    intensity = min(1.0, excess / full_intensity_threshold)

    # Apply sqrt to make gradients more visible at lower intensities
    intensity = math.sqrt(intensity)

    if deviation > 0:
        # Green: overperforming
        # From white (#ffffff) to green (#4caf50)
        r = int(255 - (255 - 76) * intensity)   # 255 -> 76
        g = int(255 - (255 - 175) * intensity)  # 255 -> 175
        b = int(255 - (255 - 80) * intensity)   # 255 -> 80
    else:
        # Red: underperforming
        # From white (#ffffff) to red (#e57373)
        r = int(255 - (255 - 229) * intensity)  # 255 -> 229
        g = int(255 - (255 - 115) * intensity)  # 255 -> 115
        b = int(255 - (255 - 115) * intensity)  # 255 -> 115

    return f'rgb({r}, {g}, {b})'


def build_h2h_grid(engines, h2h_raw):
    """
    Build the H2H grid for the dashboard.

    Args:
        engines: List of (Engine, EloRating) tuples, sorted by Elo
        h2h_raw: Raw H2H data from get_h2h_raw_data()

    Returns:
        List of row dicts with engine info, cells, and stability score
    """
    # Build lookups
    engine_elos = {e.Engine.id: float(e.EloRating.elo) for e in engines}

    grid = []
    for row_idx, row_engine in enumerate(engines):
        row_id = row_engine.Engine.id
        row_elo = engine_elos[row_id]
        row_rank = row_idx + 1

        cells = []
        # Track deviations for stability calculation
        total_excess_deviation = 0.0
        total_opponent_games = 0

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

            # Track for stability calculation (weighted by games)
            excess = calculate_excess_deviation(deviation, total_games)
            total_excess_deviation += excess * total_games
            total_opponent_games += total_games

            # Color based purely on deviation from expected:
            # - Within tolerance -> white
            # - Overperforming (positive) -> green
            # - Underperforming (negative) -> red
            bg_color = deviation_to_color(deviation, total_games)

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

        # Calculate stability score (0-100, higher = more stable/settled)
        # Average excess deviation weighted by games, then convert to 0-100
        if total_opponent_games > 0:
            avg_excess = total_excess_deviation / total_opponent_games
            # Map avg_excess to 0-100: 0 excess = 100, 2+ excess SDs = 0
            stability = max(0, 100 - (avg_excess * 50))
        else:
            stability = 0  # No games = no stability data

        stability_rounded = round(stability)
        grid.append({
            'rank': row_rank,
            'engine_name': row_engine.Engine.name,
            'engine_type': get_engine_type(row_engine.Engine.name),
            'elo': row_elo,
            'games_played': row_engine.EloRating.games_played,
            'stability': stability_rounded,
            'stability_color': stability_to_color(stability_rounded),
            'cells': cells
        })

    return grid


def get_last_played_engines():
    """
    Get the engine names from the most recently played game.
    Returns tuple of (engine1_name, engine2_name) or (None, None) if no games.
    """
    db = get_db()
    Engine, Game, EloRating = get_models()

    last_game = db.session.query(Game).order_by(Game.created_at.desc()).first()

    if not last_game:
        return (None, None)

    white_engine = db.session.query(Engine).filter(Engine.id == last_game.white_engine_id).first()
    black_engine = db.session.query(Engine).filter(Engine.id == last_game.black_engine_id).first()

    return (white_engine.name if white_engine else None, black_engine.name if black_engine else None)


def get_dashboard_data(active_only=True):
    """
    Get all data needed for the dashboard.
    Returns (engines, grid, column_headers, last_played_engines).
    """
    engines = get_engines_ranked_by_elo(active_only=active_only)

    if not engines:
        return [], [], [], (None, None)

    h2h_raw = get_h2h_raw_data()
    grid = build_h2h_grid(engines, h2h_raw)
    column_headers = [(i + 1, e.Engine.name) for i, e in enumerate(engines)]
    last_played = get_last_played_engines()

    return engines, grid, column_headers, last_played

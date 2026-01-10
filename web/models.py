"""
SQLAlchemy models for chess engine competition tracking.
"""

from datetime import datetime
from web.database import db


class Engine(db.Model):
    """Chess engine metadata."""
    __tablename__ = 'engines'

    id = db.Column(db.Integer, primary_key=True)
    name = db.Column(db.String(100), unique=True, nullable=False)
    binary_path = db.Column(db.String(500))
    active = db.Column(db.Boolean, default=True)
    initial_elo = db.Column(db.Integer, default=1500)
    uci_options = db.Column(db.JSON)
    created_at = db.Column(db.DateTime, default=datetime.utcnow)

    def __repr__(self):
        return f'<Engine {self.name}>'


class Game(db.Model):
    """Individual game result."""
    __tablename__ = 'games'

    id = db.Column(db.Integer, primary_key=True)
    white_engine_id = db.Column(db.Integer, db.ForeignKey('engines.id'), nullable=False)
    black_engine_id = db.Column(db.Integer, db.ForeignKey('engines.id'), nullable=False)
    result = db.Column(db.String(10), nullable=False)  # '1-0', '0-1', '1/2-1/2', '*'
    white_score = db.Column(db.Numeric(2, 1), nullable=False)  # 1.0, 0.5, or 0.0
    black_score = db.Column(db.Numeric(2, 1), nullable=False)
    date_played = db.Column(db.Date, nullable=False)
    time_control = db.Column(db.String(50))
    opening_name = db.Column(db.String(100))
    opening_fen = db.Column(db.Text)
    pgn = db.Column(db.Text)  # Full PGN content of the game
    created_at = db.Column(db.DateTime, default=datetime.utcnow)

    # Relationships
    white_engine = db.relationship('Engine', foreign_keys=[white_engine_id], backref='games_as_white')
    black_engine = db.relationship('Engine', foreign_keys=[black_engine_id], backref='games_as_black')

    def __repr__(self):
        return f'<Game {self.id}: {self.result}>'


class EloRating(db.Model):
    """Current Elo rating for an engine."""
    __tablename__ = 'elo_ratings'

    id = db.Column(db.Integer, primary_key=True)
    engine_id = db.Column(db.Integer, db.ForeignKey('engines.id'), nullable=False, unique=True)
    elo = db.Column(db.Numeric(7, 2), nullable=False)
    games_played = db.Column(db.Integer, default=0)
    updated_at = db.Column(db.DateTime, default=datetime.utcnow, onupdate=datetime.utcnow)

    # Relationship
    engine = db.relationship('Engine', backref=db.backref('rating', uselist=False))

    def __repr__(self):
        return f'<EloRating {self.engine.name if self.engine else "?"}: {self.elo}>'

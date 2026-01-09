#!/usr/bin/env python3
"""
Flask application for chess engine competition dashboard.
"""

import os
from pathlib import Path
from flask import Flask
from flask_sqlalchemy import SQLAlchemy
from dotenv import load_dotenv

# Load environment variables from project root
project_root = Path(__file__).parent.parent
load_dotenv(project_root / '.env')

# Initialize SQLAlchemy without app (will be configured later)
db = SQLAlchemy()


def create_app():
    """Application factory."""
    app = Flask(__name__)

    # Configuration
    app.config['SQLALCHEMY_DATABASE_URI'] = os.getenv('DATABASE_URL')
    app.config['SQLALCHEMY_TRACK_MODIFICATIONS'] = False
    app.config['SECRET_KEY'] = os.getenv('SECRET_KEY', 'dev-key-change-in-production')

    # Initialize extensions
    db.init_app(app)

    # Register routes
    from web.routes import register_routes
    register_routes(app)

    return app


# Create app instance for running directly
app = create_app()

if __name__ == '__main__':
    app.run(debug=True, port=5000)

"""
Flask routes for the competition dashboard.
"""

from flask import render_template, request
from web.queries import get_dashboard_data


def register_routes(app):
    """Register all routes with the Flask app."""

    @app.route('/')
    def dashboard():
        """Main dashboard showing H2H grid."""
        # Check for active_only parameter (default True)
        active_only = request.args.get('all') != '1'

        engines, grid, column_headers = get_dashboard_data(active_only=active_only)

        return render_template(
            'dashboard.html',
            grid=grid,
            column_headers=column_headers,
            active_only=active_only,
            total_engines=len(engines)
        )

    @app.route('/health')
    def health():
        """Health check endpoint."""
        return {'status': 'ok'}

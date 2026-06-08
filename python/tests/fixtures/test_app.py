"""Simple Flask test application for TestServer integration tests."""

from flask import Flask, jsonify


def create_app():
    """Create and configure the Flask application."""
    app = Flask(__name__)

    @app.route("/health")
    def health():
        """Health check endpoint."""
        return jsonify({"status": "healthy"})

    @app.route("/api/users")
    def get_users():
        """Get users endpoint."""
        return jsonify({
            "users": [
                {"id": 1, "name": "Alice"},
                {"id": 2, "name": "Bob"},
            ]
        })

    @app.route("/api/echo/<message>")
    def echo(message):
        """Echo endpoint."""
        return jsonify({"message": message})

    return app


# For use with TestServer.from_app()
app = create_app()


if __name__ == "__main__":
    app.run(host="127.0.0.1", port=18765)

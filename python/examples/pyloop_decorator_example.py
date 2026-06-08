"""
Example of PyLoop decorator-based HTTP server.

This example demonstrates the FastAPI-style decorator API for creating
HTTP servers with Python handlers backed by the Rust HTTP server.

Run with: python examples/pyloop_decorator_example.py
Test with:
    curl http://127.0.0.1:8000/
    curl http://127.0.0.1:8000/users/123
    curl -X POST http://127.0.0.1:8000/users -H "Content-Type: application/json" -d '{"name": "Alice"}'
"""

from cclab.orbit import App
import asyncio

app = App(title="PyLoop Demo", version="1.0.0")


@app.get("/")
async def root(request):
    """Root endpoint."""
    return {"message": "Hello from PyLoop!", "path": request["path"]}


@app.get("/users/{user_id}")
async def get_user(request):
    """Get user by ID."""
    user_id = request["path_params"]["user_id"]

    # Simulate async database query
    await asyncio.sleep(0.001)

    return {
        "user_id": user_id,
        "name": f"User {user_id}",
        "status": "active"
    }


@app.post("/users")
async def create_user(request):
    """Create a new user."""
    body = request["body"] or {}

    return {
        "id": "new_123",
        "name": body.get("name", "Unknown"),
        "created": True
    }


@app.get("/sync")
def sync_handler(request):
    """Sync handler example."""
    return {
        "sync": True,
        "timestamp": "2024-01-01",
        "method": request["method"]
    }


if __name__ == "__main__":
    app.serve(host="127.0.0.1", port=8000)

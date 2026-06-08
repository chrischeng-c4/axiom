"""
Example demonstrating graceful shutdown support in data-bridge-api.

This example shows how to:
1. Register startup and shutdown hooks
2. Use signal handlers for graceful shutdown
3. Clean up resources properly
"""
import asyncio
from cclab.quasar import App, setup_signal_handlers


# Create app with custom shutdown timeout
app = App(
    title="Shutdown Example",
    version="1.0.0",
    shutdown_timeout=10.0  # 10 second timeout for shutdown hooks
)


# Simulated database connection
class Database:
    def __init__(self):
        self.connected = False

    async def connect(self):
        print("Connecting to database...")
        await asyncio.sleep(0.1)  # Simulate connection time
        self.connected = True
        print("Database connected!")

    async def disconnect(self):
        print("Disconnecting from database...")
        await asyncio.sleep(0.1)  # Simulate cleanup time
        self.connected = False
        print("Database disconnected!")


db = Database()


# Register startup hook
@app.on_startup
async def startup():
    """Initialize resources on startup."""
    print("Application starting up...")
    await db.connect()
    print("Startup complete!")


# Register shutdown hook
@app.on_shutdown
async def shutdown():
    """Clean up resources on shutdown."""
    print("Application shutting down...")
    await db.disconnect()
    print("Shutdown complete!")


# Configure HTTP client (will be cleaned up automatically on shutdown)
app.configure_http_client(
    base_url="https://api.example.com",
    timeout=30.0
)


@app.get("/")
async def root():
    """Root endpoint."""
    return {"message": "Hello, World!", "db_connected": db.connected}


@app.get("/health")
async def health():
    """Health check endpoint."""
    if app.is_shutting_down:
        # Return 503 Service Unavailable during shutdown
        from cclab.quasar import HTTPException
        raise HTTPException(503, "Service is shutting down")

    return {
        "status": "healthy",
        "db_connected": db.connected,
        "shutting_down": app.is_shutting_down
    }


@app.get("/data")
async def get_data():
    """Example endpoint that uses the database."""
    if not db.connected:
        from cclab.quasar import HTTPException
        raise HTTPException(503, "Database not connected")

    return {"data": "some data from database"}


async def main():
    """
    Main entry point demonstrating shutdown behavior.

    In a real application, you would integrate with a ASGI server
    like uvicorn which would handle startup/shutdown automatically.
    """
    # Setup signal handlers (optional)
    # This allows Ctrl+C or kill signals to trigger graceful shutdown
    setup_signal_handlers(app)

    # Run startup hooks
    await app.startup()

    # Simulate running the application
    print("\nApplication is running. Press Ctrl+C to shutdown gracefully.\n")
    print("Routes registered:")
    for route in app.routes:
        print(f"  {route.method:6s} {route.path}")

    try:
        # In a real app, this would be replaced by the ASGI server event loop
        await asyncio.sleep(5.0)  # Simulate some runtime
        print("\nSimulating shutdown request...\n")
    except KeyboardInterrupt:
        print("\nReceived keyboard interrupt...\n")
    finally:
        # Run shutdown hooks
        await app.shutdown()
        print("\nApplication stopped.")


if __name__ == "__main__":
    asyncio.run(main())

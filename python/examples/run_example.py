"""Example showing app.run() for local development."""
from cclab.quasar import App

app = App(title="Local Dev Example", version="1.0.0")

@app.get("/")
async def root():
    """Root endpoint."""
    return {"message": "Hello from app.run()!"}

@app.get("/health")
async def health():
    """Health check."""
    return {"status": "healthy"}

if __name__ == "__main__":
    # Simple usage
    print("Starting app on http://127.0.0.1:8000")
    print("Try http://127.0.0.1:8000/docs for Swagger UI")

    # Run with defaults
    app.run()

    # Or with custom config:
    # app.run(host="0.0.0.0", port=3000, reload=True)

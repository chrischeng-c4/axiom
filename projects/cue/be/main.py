"""Cue backend entry point."""

import uvicorn


if __name__ == "__main__":
    uvicorn.run(
        "src.api.main:app",
        host="0.0.0.0",
        port=3210,
        reload=True,
        log_level="info",
    )

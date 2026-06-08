"""Structured logging with async I/O, rotation, and multiple sinks."""

import logging

try:
    from cclab._log import Logger, BoundLogger, logger
except ImportError:
    Logger = None  # type: ignore
    BoundLogger = None  # type: ignore
    logger = None  # type: ignore


def get_logger(name: str) -> logging.Logger:
    """Get a logger by name.

    Uses cclab's Rust-backed Logger when available,
    falls back to Python's standard logging.
    """
    return logging.getLogger(name)


def configure_logging(level: str = "INFO") -> None:
    """Configure basic logging."""
    logging.basicConfig(
        level=getattr(logging, level.upper(), logging.INFO),
        format="%(asctime)s %(levelname)s %(name)s: %(message)s",
    )


__all__ = [
    "Logger",
    "BoundLogger",
    "logger",
    "get_logger",
    "configure_logging",
]

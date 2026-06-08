"""CLI framework with decorator-based command definition."""

try:
    from cclab._cmd import App
except ImportError:
    pass

__all__ = [
    "App",
]

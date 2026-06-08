"""CLI framework backed by clap with rich help and auto-completion."""

try:
    from cclab._typer import App
except ImportError:
    pass

__all__ = [
    "App",
]

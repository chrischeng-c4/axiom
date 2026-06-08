"""Data visualization with SVG export."""

try:
    from cclab._plot import Chart
except ImportError:
    pass

__all__ = [
    "Chart",
]

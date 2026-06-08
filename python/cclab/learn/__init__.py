"""Machine learning and deep learning."""

try:
    from cclab._learn import ml, dl
except ImportError:
    pass

__all__ = [
    "ml",
    "dl",
]

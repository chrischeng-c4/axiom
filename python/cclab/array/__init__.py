"""N-dimensional arrays with NumPy-compatible API."""

try:
    from cclab._array import (
        ndarray, RandomState,
        array, zeros, ones, arange, linspace, full, eye, diag,
    )
except ImportError:
    pass

__all__ = [
    "ndarray",
    "RandomState",
    "array",
    "zeros",
    "ones",
    "arange",
    "linspace",
    "full",
    "eye",
    "diag",
]

"""Stats, FFT, signal processing, interpolation, and optimization."""

try:
    from cclab._sci import stats, fft, signal, spatial, sparse, integrate, ts
except ImportError:
    pass

__all__ = [
    "stats",
    "fft",
    "signal",
    "spatial",
    "sparse",
    "integrate",
    "ts",
]

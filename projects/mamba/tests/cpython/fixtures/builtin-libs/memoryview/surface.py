# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""builtin-libs/memoryview: surface probes (CPython 3.12 oracle)."""

# Probes for the documented memoryview attributes and methods on a
# 1-D bytes-backed view. Each assert verifies the API surface exists.

mv = memoryview(bytearray(b"abcd"))

# Documented data attributes.
assert mv.nbytes == 4
assert mv.format == "B"
assert mv.itemsize == 1
assert mv.ndim == 1
assert mv.shape == (4,)
assert mv.strides == (1,)
assert mv.readonly is False
assert mv.contiguous is True
assert mv.c_contiguous is True
assert mv.f_contiguous is True
assert mv.obj is not None

# Documented methods are callable / present.
assert mv.tolist() == [97, 98, 99, 100]
assert mv.tobytes() == b"abcd"
assert mv.hex() == "61626364"
assert callable(mv.cast)
assert callable(mv.release)
assert callable(mv.toreadonly)

# A read-only view derived from a writable one.
ro = mv.toreadonly()
assert ro.readonly is True
assert ro.tobytes() == b"abcd"

mv.release()
print("surface OK")

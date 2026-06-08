# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""builtin-libs/slice: surface probes (CPython 3.12 oracle)."""

# `slice` is a builtin type; verify its documented attribute/method surface.

# Read-only data attributes.
s = slice(1, 2, 3)
assert hasattr(s, "start")
assert hasattr(s, "stop")
assert hasattr(s, "step")

# The slice.indices(length) method.
assert hasattr(slice, "indices")
assert callable(slice(0, 5).indices)

# Dunder protocols that slice implements.
assert hasattr(slice, "__eq__")
assert hasattr(slice, "__hash__")
assert hasattr(slice, "__lt__")
assert hasattr(slice, "__repr__")

# slice is a type usable as a constructor.
assert isinstance(slice(5), slice)
assert isinstance(slice(1, 2), slice)
assert isinstance(slice(1, 2, 3), slice)

print("surface OK")

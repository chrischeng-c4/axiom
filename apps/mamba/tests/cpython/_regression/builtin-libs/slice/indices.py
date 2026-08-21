# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""slice: the indices(length) computation (CPython 3.12 oracle)."""

# slice.indices(length) returns (start, stop, step) clamped to a sequence
# of the given length, ready to drive a range().

# Defaults: a bare slice over the whole sequence.
assert slice(None, None, None).indices(5) == (0, 5, 1)

# Positive step, out-of-range stop is clamped to length.
assert slice(0, 100).indices(5) == (0, 5, 1)

# Negative start counts from the end.
assert slice(-3, None).indices(5) == (2, 5, 1)

# Negative step walks backwards; the "before the start" sentinel is -1.
assert slice(None, None, -1).indices(5) == (4, -1, -1)

# Explicit positive step.
assert slice(0, 5, 2).indices(5) == (0, 5, 2)

# indices() drives an equivalent index walk over a real sequence.
data = list(range(10))
for sl in (slice(2, 8), slice(None, None, 2), slice(None, None, -1), slice(-4, -1)):
    start, stop, step = sl.indices(len(data))
    assert [data[i] for i in range(start, stop, step)] == data[sl]

# indices() accepts objects with __index__ for the length.
class Len:
    def __index__(self):
        return 5
assert slice(1, 4).indices(Len()) == (1, 4, 1)

print("indices OK")

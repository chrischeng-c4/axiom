# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""builtin-libs/memoryview: core behavior asserts (CPython 3.12 oracle)."""

# Indexing returns the integer element value.
mv = memoryview(b"hello")
assert mv[0] == ord("h")
assert mv[-1] == ord("o")
assert len(mv) == 5

# Slicing yields a new sub-view sharing the buffer.
assert bytes(mv[1:4]) == b"ell"
assert bytes(mv[::-1]) == b"olleh"

# Iteration and membership operate on element ints.
assert list(mv) == [104, 101, 108, 108, 111]
assert ord("h") in mv
assert 999 not in mv

# Equality compares contents against other buffers.
assert mv == b"hello"
assert mv == memoryview(b"hello")
assert mv != b"world"

# tolist / tobytes / hex round-trips.
assert mv.tolist() == [104, 101, 108, 108, 111]
assert mv.tobytes() == b"hello"
assert mv.hex() == "68656c6c6f"

# Writable view writes through to the backing buffer.
buf = bytearray(b"hello")
w = memoryview(buf)
w[0] = ord("H")
w[1:5] = b"ELLO"
assert buf == b"HELLO"

# Whole-slice assignment of equal length.
w[:] = b"world"
assert buf == b"world"

# .obj exposes the exporter; .toreadonly mirrors content read-only.
assert w.obj is buf
ro = w.toreadonly()
assert ro.tobytes() == b"world"

# Context-manager protocol releases the view on exit.
with memoryview(b"abc") as cm:
    assert cm[0] == ord("a")

print("behavior OK")

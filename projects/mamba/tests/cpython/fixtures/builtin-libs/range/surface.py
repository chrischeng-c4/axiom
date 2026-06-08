# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""range: API surface probes (CPython 3.12 oracle)."""

# range is a builtin type exposing read-only start/stop/step ints plus
# the count() and index() sequence methods.
r = range(2, 20, 3)
assert isinstance(r, range)
assert type(r) is range

# Read-only attributes carry the constructor arguments.
assert r.start == 2
assert r.stop == 20
assert r.step == 3
assert type(r.start) is int

# A single-argument range fills in start=0, step=1.
one = range(5)
assert one.start == 0
assert one.stop == 5
assert one.step == 1

# Sequence methods exist and are callable.
assert callable(r.count)
assert callable(r.index)

# range supports len(), indexing, slicing, membership, and iteration.
assert len(range(10)) == 10
assert range(10)[3] == 3
assert range(10)[2:5] == range(2, 5)
assert 4 in range(10)
assert list(iter(range(3))) == [0, 1, 2]

# iter(range) and reversed(range) yield distinct iterator types, both of
# which are themselves iterators (iter(it) is it).
fwd = iter(range(3))
rev = reversed(range(3))
assert iter(fwd) is fwd
assert iter(rev) is rev
assert hasattr(fwd, "__next__")
assert hasattr(rev, "__next__")

print("surface OK")

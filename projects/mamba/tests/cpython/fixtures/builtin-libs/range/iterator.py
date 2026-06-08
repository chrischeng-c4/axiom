# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""range: forward/reverse iterator behavior and state."""


# A fresh iterator yields every element once, then stays exhausted.
it = iter(range(3))
assert next(it) == 0
assert next(it) == 1
assert next(it) == 2
try:
    next(it)
    raise AssertionError("expected StopIteration")
except StopIteration:
    pass
# Re-pulling an exhausted iterator keeps raising / produces nothing.
assert list(it) == []

# reversed(range) is a distinct iterator walking elements backwards.
it = reversed(range(10, 20, 2))
assert list(it) == [18, 16, 14, 12, 10]

# A range is re-iterable: each iter() call starts over.
r = range(4)
assert list(r) == [0, 1, 2, 3]
assert list(r) == [0, 1, 2, 3]
first = iter(r)
second = iter(r)
assert next(first) == 0
assert next(second) == 0  # independent cursors

# __setstate__ on a forward iterator fast-forwards by an index count.
it = iter(range(10, 20, 2))
it.__setstate__(2)
assert list(it) == [14, 16, 18]

# __setstate__ on a reverse iterator likewise advances it.
it = reversed(range(10, 20, 2))
it.__setstate__(3)
assert list(it) == [12, 10]

# State also works against bignum-bounded ranges.
it = iter(range(-(2**65), 20, 2))
it.__setstate__(2**64 + 7)
assert list(it) == [14, 16, 18]

# The internal range-iterator types are not directly constructible.
fwd_type = type(iter(range(0)))
rev_type = type(reversed(range(0)))
for t in (fwd_type, rev_type):
    try:
        t(1, 3, 1)
        raise AssertionError("expected TypeError")
    except TypeError:
        pass

# Reverse-iterating equals reversing the materialized list, for many shapes.
import sys

shapes = [range(10), range(0), range(1, 9, 3), range(8, 0, -3),
          range(sys.maxsize + 1, sys.maxsize + 6)]
for r in shapes:
    assert list(reversed(r)) == list(r)[::-1]

print("iterator OK")

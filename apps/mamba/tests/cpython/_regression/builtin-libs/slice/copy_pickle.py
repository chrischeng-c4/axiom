# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""slice: copy / deepcopy / pickle round-trips (CPython 3.12 oracle)."""

import copy
import pickle

# slice is immutable, so a shallow copy returns the very same object.
s = slice(1, 10)
assert copy.copy(s) is s
s = slice(1, 10, 2)
assert copy.copy(s) is s

# With mutable members, copy.copy still returns the same slice object,
# so the member objects are shared (same identity).
s = slice([1, 2], [3, 4], [5, 6])
c = copy.copy(s)
assert s is c
assert s.start is c.start and s.stop is c.stop and s.step is c.step

# deepcopy produces a new, equal slice whose mutable members are clones.
d = copy.deepcopy(s)
assert d is not s
assert d == s
assert d.start is not s.start
assert d.stop is not s.stop
assert d.step is not s.step

# pickle round-trips at every protocol: equal value, fresh identity.
orig = slice(10, 20, 3)
for protocol in range(pickle.HIGHEST_PROTOCOL + 1):
    t = pickle.loads(pickle.dumps(orig, protocol))
    assert t == orig
    assert t is not orig
    assert t.indices(15) == orig.indices(15)

print("copy_pickle OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""set_methods: construction from iterables, iteration, and deepcopy."""

import copy

# set() consumes any iterable: list, tuple, dict (keys), range, str, generator.
def gen():
    yield 1
    yield 2
    yield 2

assert set([1, 2, 3]) == {1, 2, 3}
assert set((1, 2, 3)) == {1, 2, 3}
assert set({"one": 1, "two": 2}) == {"one", "two"}   # dict -> its keys
assert set(range(3)) == {0, 1, 2}
assert set("abca") == {"a", "b", "c"}
assert set(gen()) == {1, 2}

# An exception raised inside the source iterator propagates out of set().
def bad():
    yield 1
    raise ValueError("boom")

try:
    set(bad())
    raise AssertionError("expected ValueError")
except ValueError:
    pass

# Iterating then mutating the same set raises RuntimeError (size changed).
s = {1, 2, 3}
try:
    for _ in s:
        s.add(99)
    raise AssertionError("expected RuntimeError")
except RuntimeError:
    pass

# Membership over a frozenset of single chars agrees with a dict's keys.
fs = frozenset("simsalabim")
keys = dict.fromkeys("simsalabim")
for ch in "abcdefghijklmnopqrstuvwxyz":
    assert (ch in fs) == (ch in keys)

# deepcopy produces an equal but independent set of immutable elements.
original = {(1, 2), (3, 4), frozenset([5])}
clone = copy.deepcopy(original)
assert clone == original
assert clone is not original

print("iteration_and_construction OK")

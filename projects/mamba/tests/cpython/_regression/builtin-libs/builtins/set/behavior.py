"""Behavior contract for builtins.set.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

# Rule 1: set() with no args returns empty set
assert set() == set(), f"set() empty check"
assert len(set()) == 0, f"len(set()) = {len(set())!r}"

# Rule 2: set(iterable)
s = set([1, 2, 3])
assert s == {1, 2, 3}, f"set([1,2,3]) = {s!r}"
s = set("abc")
assert s == {"a", "b", "c"}, f"set('abc') = {s!r}"

# Rule 3: deduplication
s = set([1, 2, 2, 3, 3, 3])
assert s == {1, 2, 3}, f"dedup = {s!r}"
assert len(s) == 3, f"dedup len = {len(s)!r}"

# Rule 4: add
s = {1, 2}
s.add(3)
assert 3 in s, "add(3) failed"
s.add(2)  # existing element, no-op
assert len(s) == 3, f"add existing changed len = {len(s)!r}"

# Rule 5: discard (no error if missing)
s = {1, 2, 3}
s.discard(2)
assert s == {1, 3}, f"discard = {s!r}"
s.discard(99)  # no-op
assert s == {1, 3}, f"discard missing = {s!r}"

# Rule 6: remove (raises KeyError if missing)
s = {1, 2, 3}
s.remove(2)
assert s == {1, 3}, f"remove = {s!r}"
_raised = False
try:
    s.remove(99)
except KeyError:
    _raised = True
assert _raised, "remove(99) did not raise KeyError"

# Rule 7: in / not in
s = {1, 2, 3}
assert 2 in s, "2 in s failed"
assert 99 not in s, "99 not in s failed"

# Rule 8: union
a = {1, 2}
b = {2, 3}
assert a | b == {1, 2, 3}, f"union = {a | b!r}"
assert a.union(b) == {1, 2, 3}, f"union method = {a.union(b)!r}"

# Rule 9: intersection
assert a & b == {2}, f"intersection = {a & b!r}"
assert a.intersection(b) == {2}, f"intersection method = {a.intersection(b)!r}"

# Rule 10: difference
assert a - b == {1}, f"difference = {a - b!r}"
assert a.difference(b) == {1}, f"difference method = {a.difference(b)!r}"

# Rule 11: symmetric_difference
assert a ^ b == {1, 3}, f"symmetric_difference = {a ^ b!r}"

# Rule 12: issubset / issuperset
assert {1, 2}.issubset({1, 2, 3}), "issubset failed"
assert {1, 2, 3}.issuperset({1, 2}), "issuperset failed"
assert not {1, 2}.issuperset({1, 2, 3}), "issuperset wrong"

# Rule 13: isdisjoint
assert {1, 2}.isdisjoint({3, 4}), "isdisjoint failed"
assert not {1, 2}.isdisjoint({2, 3}), "isdisjoint wrong"

# Rule 14: set is not ordered — no indexing
_raised = False
try:
    s = {1, 2, 3}
    _ = s[0]  # type: ignore[index]
except TypeError:
    _raised = True
assert _raised, "set[0] did not raise TypeError"

# Rule 15: clear
s = {1, 2, 3}
s.clear()
assert s == set(), f"clear = {s!r}"

print("behavior OK")

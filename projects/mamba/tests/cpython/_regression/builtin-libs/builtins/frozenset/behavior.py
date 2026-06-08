"""Behavior contract for builtins.frozenset.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

# Rule 1: frozenset() with no args returns empty frozenset
fs = frozenset()
assert fs == frozenset(), f"frozenset() empty"
assert len(fs) == 0, f"len(frozenset()) = {len(fs)!r}"

# Rule 2: frozenset(iterable)
fs = frozenset([1, 2, 3])
assert fs == frozenset({1, 2, 3}), f"frozenset([1,2,3]) = {fs!r}"

# Rule 3: deduplication
fs = frozenset([1, 2, 2, 3])
assert len(fs) == 3, f"dedup len = {len(fs)!r}"

# Rule 4: in / not in
fs = frozenset([1, 2, 3])
assert 2 in fs, "2 in fs failed"
assert 9 not in fs, "9 not in fs failed"

# Rule 5: immutability — no add
_raised = False
try:
    fs.add(4)  # type: ignore[attr-defined]
except AttributeError:
    _raised = True
assert _raised, "frozenset.add did not raise AttributeError"

# Rule 6: hashable — can be used as dict key
d = {frozenset([1, 2]): "pair"}
assert d[frozenset([1, 2])] == "pair", "frozenset as dict key failed"

# Rule 7: hashable — can be in a set
s = {frozenset([1]), frozenset([2])}
assert len(s) == 2, f"frozenset in set = {s!r}"

# Rule 8: set operations
a = frozenset([1, 2, 3])
b = frozenset([2, 3, 4])
assert a | b == frozenset([1, 2, 3, 4]), f"union = {a | b!r}"
assert a & b == frozenset([2, 3]), f"intersection = {a & b!r}"
assert a - b == frozenset([1]), f"difference = {a - b!r}"
assert a ^ b == frozenset([1, 4]), f"symmetric_diff = {a ^ b!r}"

# Rule 9: result of set ops is frozenset
assert type(a | b) is frozenset, f"type(a|b) = {type(a|b).__name__!r}"
assert type(a & b) is frozenset, f"type(a&b) = {type(a&b).__name__!r}"

# Rule 10: issubset / issuperset
assert frozenset([1, 2]).issubset(frozenset([1, 2, 3])), "issubset failed"
assert frozenset([1, 2, 3]).issuperset(frozenset([1, 2])), "issuperset failed"

# Rule 11: isdisjoint
assert frozenset([1, 2]).isdisjoint(frozenset([3, 4])), "isdisjoint failed"

# Rule 12: frozenset == set comparison
assert frozenset([1, 2, 3]) == {1, 2, 3}, "frozenset == set failed"

print("behavior OK")

# Operational AssertionPass seed for `collections.OrderedDict`.
# Surface: insertion-order preservation across keys/values/items;
# indexing; `in` containment; `get` with and without default; `len`;
# iteration preserves insertion order; re-assigning an existing key
# updates value but keeps the key's original position; construction
# from an iterable of pairs; empty OrderedDict.
#
# NOT exercised here (broken on mamba 0.3.60): `del od[k]` (the key
# remains visible); `bool()` of an empty OrderedDict (returns True
# instead of False); `move_to_end` / `popitem` (per the deque seed
# notes). Those gaps move to spec/.
import collections
_ledger: list[int] = []

# Insertion order is preserved across keys/values/items
od = collections.OrderedDict()
od["a"] = 1
od["b"] = 2
od["c"] = 3
assert list(od.keys()) == ["a", "b", "c"]; _ledger.append(1)
assert list(od.values()) == [1, 2, 3]; _ledger.append(1)
assert list(od.items()) == [("a", 1), ("b", 2), ("c", 3)]; _ledger.append(1)

# Indexing returns the stored value
assert od["a"] == 1; _ledger.append(1)
assert od["b"] == 2; _ledger.append(1)
assert od["c"] == 3; _ledger.append(1)

# len() reports the number of entries
assert len(od) == 3; _ledger.append(1)

# `in` / `not in` containment
assert "a" in od; _ledger.append(1)
assert "b" in od; _ledger.append(1)
assert "z" not in od; _ledger.append(1)

# .get returns the value if present, None / default otherwise
assert od.get("a") == 1; _ledger.append(1)
assert od.get("missing") is None; _ledger.append(1)
assert od.get("missing", 99) == 99; _ledger.append(1)
assert od.get("a", 99) == 1; _ledger.append(1)  # present trumps default

# Construction from a list of pairs
od2 = collections.OrderedDict([("x", 10), ("y", 20), ("z", 30)])
assert list(od2.keys()) == ["x", "y", "z"]; _ledger.append(1)
assert list(od2.values()) == [10, 20, 30]; _ledger.append(1)
assert od2["x"] == 10; _ledger.append(1)
assert od2["y"] == 20; _ledger.append(1)
assert od2["z"] == 30; _ledger.append(1)
assert len(od2) == 3; _ledger.append(1)

# Iteration yields keys in insertion order
collected = []
for k in od:
    collected.append(k)
assert collected == ["a", "b", "c"]; _ledger.append(1)

# Re-assigning an existing key updates the value but preserves
# the key's original position (no move-to-end on update)
od["b"] = 999
assert list(od.keys()) == ["a", "b", "c"]; _ledger.append(1)
assert od["b"] == 999; _ledger.append(1)
# The other keys still hold their original values
assert od["a"] == 1; _ledger.append(1)
assert od["c"] == 3; _ledger.append(1)

# Empty OrderedDict
od_empty = collections.OrderedDict()
assert len(od_empty) == 0; _ledger.append(1)
assert list(od_empty.keys()) == []; _ledger.append(1)
assert list(od_empty.values()) == []; _ledger.append(1)
assert list(od_empty.items()) == []; _ledger.append(1)
assert "anything" not in od_empty; _ledger.append(1)
assert od_empty.get("missing") is None; _ledger.append(1)

# bool() — non-empty OrderedDict is truthy
assert bool(od); _ledger.append(1)

# Insertion preserves later additions at the end
od3 = collections.OrderedDict()
od3["first"] = 1
od3["second"] = 2
od3["third"] = 3
od3["fourth"] = 4
assert list(od3.keys()) == ["first", "second", "third", "fourth"]; _ledger.append(1)
assert list(od3.values()) == [1, 2, 3, 4]; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_ordered_dict_ops {sum(_ledger)} asserts")

# Operational AssertionPass seed for advanced dict surfaces beyond
# test_dict_ops (literal/get/update/pop/setdefault/comprehension).
# Surface: popitem LIFO order, pop with default on a missing key,
# update via kwargs, dict.fromkeys classmethod, shallow copy via
# .copy, .items() yielding (k, v) tuples that compare equal as a
# sorted list.
_ledger: list[int] = []

# popitem removes the LAST inserted item (LIFO in 3.7+)
d = {"a": 1, "b": 2, "c": 3}
last = d.popitem()
assert last == ("c", 3); _ledger.append(1)
# popitem actually mutated the dict — len drops
assert len(d) == 2; _ledger.append(1)

# pop with default returns the default when the key is absent
v = d.pop("missing", "ms")
assert v == "ms"; _ledger.append(1)
# d is unchanged when pop falls through to default
assert len(d) == 2; _ledger.append(1)

# update via keyword arguments — kw=val pairs land as string keys
e: dict = {}
e.update(a=1, b=2)
assert e["a"] == 1; _ledger.append(1)
assert e["b"] == 2; _ledger.append(1)

# kwargs update overwrites overlapping keys
e.update(a=99)
assert e["a"] == 99; _ledger.append(1)

# dict.fromkeys creates a dict with the iterable as keys and a single
# shared default value
f = dict.fromkeys(["x", "y", "z"], 0)
assert f == {"x": 0, "y": 0, "z": 0}; _ledger.append(1)
# default value defaults to None when omitted
g = dict.fromkeys(["a"])
assert g["a"] is None; _ledger.append(1)

# .copy returns a shallow copy that is independent at the top level
h = {"x": 1, "y": 2}
i = h.copy()
i["new"] = 99
assert "new" not in h; _ledger.append(1)
assert h == {"x": 1, "y": 2}; _ledger.append(1)

# .items() yields (k, v) tuples — sortable into a deterministic list
j = {"x": 1, "y": 2}
assert sorted(j.items()) == [("x", 1), ("y", 2)]; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_dict_advanced_ops {sum(_ledger)} asserts")

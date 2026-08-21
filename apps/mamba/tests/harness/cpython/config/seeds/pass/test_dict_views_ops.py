# Operational AssertionPass seed for dict view objects beyond
# test_dict_ops basics.
# Surface: .keys() / .values() / .items() materialize to expected
# lists; len() of each view matches len(dict); `in` over .keys() /
# .values() / .items(); iteration order preserves insertion order;
# dict-vs-dict equality is order-insensitive; inequality on
# different values; .get with present key / missing key + default /
# missing key without default returns None.
# Note: live-mutation propagation through cached view objects and
# set-algebra on dict-key views (& | - ^) are NOT exercised here —
# both have known breakages on the current mamba runtime.
_ledger: list[int] = []

d = {"a": 1, "b": 2, "c": 3}

# .keys() yields the keys in insertion order
assert list(d.keys()) == ["a", "b", "c"]; _ledger.append(1)
# .values() yields the values in insertion order
assert list(d.values()) == [1, 2, 3]; _ledger.append(1)
# .items() yields (key, value) tuples in insertion order
assert list(d.items()) == [("a", 1), ("b", 2), ("c", 3)]; _ledger.append(1)

# len() of each view equals len(dict)
assert len(d.keys()) == 3; _ledger.append(1)
assert len(d.values()) == 3; _ledger.append(1)
assert len(d.items()) == 3; _ledger.append(1)

# Membership check over .keys() is equivalent to `k in d`
assert ("a" in d.keys()) == True; _ledger.append(1)
assert ("z" in d.keys()) == False; _ledger.append(1)

# Membership check over .values()
assert (2 in d.values()) == True; _ledger.append(1)
assert (99 in d.values()) == False; _ledger.append(1)

# Membership check over .items() requires full (k, v) match
assert (("a", 1) in d.items()) == True; _ledger.append(1)
assert (("a", 99) in d.items()) == False; _ledger.append(1)

# Iterating a dict yields keys in insertion order
order: list[str] = []
for k in {"z": 1, "y": 2, "a": 3}:
    order.append(k)
assert order == ["z", "y", "a"]; _ledger.append(1)

# Dict equality is order-insensitive: same key/value pairs regardless of order
assert {"a": 1, "b": 2} == {"b": 2, "a": 1}; _ledger.append(1)
# Different values: inequality
assert {"a": 1} != {"a": 2}; _ledger.append(1)
# Different keys: inequality
assert {"a": 1} != {"b": 1}; _ledger.append(1)

# .get returns the default for a missing key
assert d.get("nope", "fallback") == "fallback"; _ledger.append(1)
# .get returns the value for a present key
assert d.get("a") == 1; _ledger.append(1)
# .get with no default returns None for a missing key
assert d.get("nope") is None; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_dict_views_ops {sum(_ledger)} asserts")

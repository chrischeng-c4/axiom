# Operational AssertionPass seed for dict mutation surfaces not
# already covered by `lang_dict_methods`, `lang_dict_merge`,
# `lang_pep584_dict_union`, `lang_dict_view_membership`,
# `lang_augmented_assignment`, or `test_dict_advanced_ops`. This seed
# asserts: `dict.setdefault(k, default)` inserts the default and
# returns it when `k` is absent, but returns the existing value
# (and leaves it unmodified) when `k` is present; `dict.setdefault`
# without a default uses None as the implicit default; `dict.update`
# accepts keyword arguments and merges them into the dict; `update`
# accepts an iterable of (key, value) pairs; `update` overrides
# existing keys with new values; constructing a dict from
# `zip(keys, values)` mirrors `dict(zip(...))` shape; constructing
# from a list of pairs preserves all distinct keys; dict union `|`
# right-side wins on collision; `|=` mutates the left dict in place;
# dict comprehension swapping {k: v} -> {v: k}; dict comprehension
# filtering by value predicate; dict.copy returns a shallow copy that
# is `==` but not `is`; dict.clear empties the dict in-place; `del
# d[k]` and `d.pop(k, default)` behave correctly with both present
# and missing keys.
_ledger: list[int] = []

# dict.setdefault — insert default on miss, leave existing value
d = {}
v = d.setdefault("k", 10)
assert v == 10; _ledger.append(1)
assert d == {"k": 10}; _ledger.append(1)

v2 = d.setdefault("k", 999)
assert v2 == 10; _ledger.append(1)
assert d == {"k": 10}; _ledger.append(1)

# setdefault keeps the original mapping intact
d["k"] = 7
assert d.setdefault("k", 42) == 7; _ledger.append(1)
assert d == {"k": 7}; _ledger.append(1)

# Implicit-None default on miss
d2 = {}
assert d2.setdefault("missing") is None; _ledger.append(1)
assert d2 == {"missing": None}; _ledger.append(1)

# Existing key with implicit-None default — returns existing
d3 = {"a": 5}
assert d3.setdefault("a") == 5; _ledger.append(1)
assert d3 == {"a": 5}; _ledger.append(1)

# Multiple setdefault chaining
counts = {}
for ch in "banana":
    counts.setdefault(ch, 0)
    counts[ch] += 1
assert counts == {"b": 1, "a": 3, "n": 2}; _ledger.append(1)

# dict.update with kwargs merges them in
u = {"x": 1}
u.update(y=2, z=3)
assert u == {"x": 1, "y": 2, "z": 3}; _ledger.append(1)

# update with kwargs overwrites existing key
u2 = {"x": 1, "y": 2}
u2.update(y=20, z=3)
assert u2 == {"x": 1, "y": 20, "z": 3}; _ledger.append(1)

# update with iterable of pairs
u3 = {"a": 1}
u3.update([("b", 2), ("c", 3)])
assert u3 == {"a": 1, "b": 2, "c": 3}; _ledger.append(1)

# update with dict argument (positional) overrides
u4 = {"a": 1, "b": 2}
u4.update({"b": 20, "c": 3})
assert u4 == {"a": 1, "b": 20, "c": 3}; _ledger.append(1)

# dict() constructor — kwargs form
assert dict(a=1, b=2) == {"a": 1, "b": 2}; _ledger.append(1)
assert dict() == {}; _ledger.append(1)

# dict() from list of pairs
assert dict([("a", 1), ("b", 2)]) == {"a": 1, "b": 2}; _ledger.append(1)
assert dict([("k", 10)]) == {"k": 10}; _ledger.append(1)
assert dict([]) == {}; _ledger.append(1)

# dict() from zip
assert dict(zip(["a", "b", "c"], [1, 2, 3])) == {"a": 1, "b": 2, "c": 3}; _ledger.append(1)
assert dict(zip(["x"], [42])) == {"x": 42}; _ledger.append(1)

# zip strict means equal-length only; mismatched lengths truncate
# without strict
assert dict(zip(["a", "b"], [1, 2, 3])) == {"a": 1, "b": 2}; _ledger.append(1)
assert dict(zip(["a", "b", "c"], [1, 2])) == {"a": 1, "b": 2}; _ledger.append(1)

# dict | dict — RHS wins on collision (PEP 584)
a = {"x": 1, "y": 2}
b = {"y": 20, "z": 30}
assert a | b == {"x": 1, "y": 20, "z": 30}; _ledger.append(1)
assert b | a == {"x": 1, "y": 2, "z": 30}; _ledger.append(1)
# Original dicts unchanged
assert a == {"x": 1, "y": 2}; _ledger.append(1)
assert b == {"y": 20, "z": 30}; _ledger.append(1)

# |= mutates left dict in place (PEP 584)
c = {"a": 1, "b": 2}
c |= {"b": 20, "c": 3}
assert c == {"a": 1, "b": 20, "c": 3}; _ledger.append(1)

# Dict comprehension swap keys <-> values
swap = {v: k for k, v in {"a": 1, "b": 2, "c": 3}.items()}
assert swap == {1: "a", 2: "b", 3: "c"}; _ledger.append(1)

# Dict comprehension filter by value predicate
src = {"a": 1, "b": 2, "c": 3, "d": 4}
evens = {k: v for k, v in src.items() if v % 2 == 0}
assert evens == {"b": 2, "d": 4}; _ledger.append(1)

# Dict comprehension building square map
squares = {x: x * x for x in range(5)}
assert squares == {0: 0, 1: 1, 2: 4, 3: 9, 4: 16}; _ledger.append(1)

# Dict comprehension from zip iterable
zd = {k: v for k, v in zip(["x", "y", "z"], [10, 20, 30])}
assert zd == {"x": 10, "y": 20, "z": 30}; _ledger.append(1)

# dict.copy — shallow copy returns equal dict
orig = {"a": 1, "b": 2}
shallow = orig.copy()
assert shallow == orig; _ledger.append(1)
# Mutating one does not affect the other
shallow["c"] = 3
assert "c" in shallow; _ledger.append(1)
assert "c" not in orig; _ledger.append(1)

# dict.clear empties the dict in place
cl = {"a": 1, "b": 2}
cl.clear()
assert cl == {}; _ledger.append(1)
assert len(cl) == 0; _ledger.append(1)

# del d[k] removes the key
dd = {"a": 1, "b": 2, "c": 3}
del dd["b"]
assert dd == {"a": 1, "c": 3}; _ledger.append(1)

# dict.pop(k) on present key
dp = {"a": 1, "b": 2}
v = dp.pop("a")
assert v == 1; _ledger.append(1)
assert dp == {"b": 2}; _ledger.append(1)

# dict.pop(k, default) returns default on miss
dp2 = {"a": 1}
assert dp2.pop("missing", "fallback") == "fallback"; _ledger.append(1)
assert dp2 == {"a": 1}; _ledger.append(1)

# dict.pop(k, default) returns existing value on hit
dp3 = {"a": 1}
assert dp3.pop("a", "fallback") == 1; _ledger.append(1)
assert dp3 == {}; _ledger.append(1)

# dict.fromkeys with shared default value
assert dict.fromkeys(["a", "b", "c"], 0) == {"a": 0, "b": 0, "c": 0}; _ledger.append(1)
assert dict.fromkeys([]) == {}; _ledger.append(1)
assert dict.fromkeys(["x"]) == {"x": None}; _ledger.append(1)

# fromkeys with mixed-type keys from an iterable
assert dict.fromkeys("abc", 1) == {"a": 1, "b": 1, "c": 1}; _ledger.append(1)

# Iteration order preserved: insertion order
io = {}
for ch in "edcba":
    io[ch] = ord(ch)
assert list(io.keys()) == ["e", "d", "c", "b", "a"]; _ledger.append(1)
assert list(io.values()) == [101, 100, 99, 98, 97]; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_dict_setdefault_kwargs_update_ops {sum(_ledger)} asserts")

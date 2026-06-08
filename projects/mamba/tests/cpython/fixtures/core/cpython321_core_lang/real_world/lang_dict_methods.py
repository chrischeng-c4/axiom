# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_dict_methods"
# subject = "cpython321.lang_dict_methods"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/lang_dict_methods.py"
# status = "filled"
# ///
"""cpython321.lang_dict_methods: execute CPython 3.12 seed lang_dict_methods"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for dict method surfaces.
# Surface: get with default and missing-→-None fallback; setdefault
# (inserts on miss, returns existing on hit); update from another dict
# and from kwargs; pop (with and without default); popitem returns a
# (key, value) tuple; keys/values/items projections; clear (empty in
# place); copy (top-level independent); membership (in / not in); len;
# dict.fromkeys constructor; subscript assign and del; nested dict
# read+write; dict comprehension; dict() from list-of-pairs and from
# kwargs; PEP 584 merge operator `|` (with override on collision).
_ledger: list[int] = []

# get — hit, default-on-miss, None-on-miss
d = {"a": 1, "b": 2}
assert d.get("a") == 1; _ledger.append(1)
assert d.get("z", 99) == 99; _ledger.append(1)
assert d.get("z") == None; _ledger.append(1)

# setdefault — inserts on miss and returns the new value; returns
# the existing value (without overwriting) on hit
assert d.setdefault("c", 3) == 3; _ledger.append(1)
assert d == {"a": 1, "b": 2, "c": 3}; _ledger.append(1)
assert d.setdefault("a", 99) == 1; _ledger.append(1)
assert d == {"a": 1, "b": 2, "c": 3}; _ledger.append(1)

# update — from another dict and from keyword arguments
e = {"x": 1}
e.update({"y": 2})
assert e == {"x": 1, "y": 2}; _ledger.append(1)
e.update(z=3)
assert e == {"x": 1, "y": 2, "z": 3}; _ledger.append(1)

# pop — by key (returns the popped value); pop with default on miss
assert e.pop("x") == 1; _ledger.append(1)
assert e.pop("missing", 99) == 99; _ledger.append(1)

# popitem — returns an arbitrary (key, value) pair (here only one
# item is present, so it's deterministic)
f = {"k": 1}
item = f.popitem()
assert item == ("k", 1); _ledger.append(1)
assert f == {}; _ledger.append(1)

# keys / values / items projections (materialized via list())
g = {"a": 1, "b": 2, "c": 3}
assert list(g.keys()) == ["a", "b", "c"]; _ledger.append(1)
assert list(g.values()) == [1, 2, 3]; _ledger.append(1)
assert list(g.items()) == [("a", 1), ("b", 2), ("c", 3)]; _ledger.append(1)

# clear — empty the dict in place
g.clear()
assert g == {}; _ledger.append(1)
assert len(g) == 0; _ledger.append(1)

# copy — top-level independent
h = {"a": 1, "b": 2}
i = h.copy()
i["c"] = 3
assert h == {"a": 1, "b": 2}; _ledger.append(1)
assert i == {"a": 1, "b": 2, "c": 3}; _ledger.append(1)

# Membership — `in` and `not in` check keys, not values
assert "a" in {"a": 1}; _ledger.append(1)
assert "z" not in {"a": 1}; _ledger.append(1)

# len — number of keys
assert len({"a": 1, "b": 2}) == 2; _ledger.append(1)
assert len({}) == 0; _ledger.append(1)

# dict.fromkeys — build a dict from an iterable of keys with a shared
# default value
assert dict.fromkeys(["a", "b"], 0) == {"a": 0, "b": 0}; _ledger.append(1)
assert dict.fromkeys(["x"], 42) == {"x": 42}; _ledger.append(1)

# Subscript assign and del
j = {}
j["x"] = 1
assert j == {"x": 1}; _ledger.append(1)
j["y"] = 2
assert j == {"x": 1, "y": 2}; _ledger.append(1)
del j["x"]
assert j == {"y": 2}; _ledger.append(1)

# Nested dict — read and write through chained subscripts
nested = {"a": {"b": 1}}
assert nested["a"]["b"] == 1; _ledger.append(1)
nested["a"]["c"] = 2
assert nested == {"a": {"b": 1, "c": 2}}; _ledger.append(1)

# Dict comprehension
comp = {k: k * 2 for k in [1, 2, 3]}
assert comp == {1: 2, 2: 4, 3: 6}; _ledger.append(1)

# dict() constructor — from a list of (key, value) pairs and from
# keyword arguments
assert dict([("a", 1), ("b", 2)]) == {"a": 1, "b": 2}; _ledger.append(1)
assert dict(a=1, b=2) == {"a": 1, "b": 2}; _ledger.append(1)

# PEP 584 — dict merge operator `|` (introduced in Python 3.9)
merged = {"a": 1} | {"b": 2}
assert merged == {"a": 1, "b": 2}; _ledger.append(1)
# On collision, the right-hand side wins
assert ({"a": 1} | {"a": 99}) == {"a": 99}; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: lang_dict_methods {sum(_ledger)} asserts")

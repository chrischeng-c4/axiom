# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "cpython321_builtins"
# dimension = "real_world"
# case = "test_defaultdict_ops"
# subject = "cpython321.test_defaultdict_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_defaultdict_ops.py"
# status = "filled"
# ///
"""cpython321.test_defaultdict_ops: execute CPython 3.12 seed test_defaultdict_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for collections.defaultdict.
# Surface: defaultdict(int) auto-fills missing keys with 0 so
# `d[k] += 1` works without explicit initialization; defaultdict(list)
# auto-fills with an empty list so `d[k].append(x)` works on first
# touch; reading a missing key materializes the default and inserts
# it into the dict; standard dict ops (len, keys, in / not in, get,
# dict()) work on a defaultdict.
from collections import defaultdict
_ledger: list[int] = []

# defaultdict(int) — missing keys default to 0
d = defaultdict(int)
d["a"] += 1
d["a"] += 1
d["b"] += 5
assert d["a"] == 2; _ledger.append(1)
assert d["b"] == 5; _ledger.append(1)

# Reading a missing key returns 0 AND inserts the default value
assert d["c"] == 0; _ledger.append(1)
# After the read, "c" is now a real key
assert "c" in d; _ledger.append(1)

# dict(defaultdict) materializes the populated entries
expected = {"a": 2, "b": 5, "c": 0}
assert dict(d) == expected; _ledger.append(1)

# defaultdict(list) — missing keys default to []
dl = defaultdict(list)
dl["x"].append(1)
dl["x"].append(2)
dl["y"].append(10)
assert dl["x"] == [1, 2]; _ledger.append(1)
assert dl["y"] == [10]; _ledger.append(1)

# Reading a missing key materializes an empty list and inserts it
empty = dl["z"]
assert empty == []; _ledger.append(1)
assert "z" in dl; _ledger.append(1)

# defaultdict supports the standard dict ops
assert len(dl) == 3; _ledger.append(1)
assert sorted(dl.keys()) == ["x", "y", "z"]; _ledger.append(1)

# in / not in
assert ("x" in dl) == True; _ledger.append(1)
assert ("missing" not in dl) == True; _ledger.append(1)

# .get with a default does NOT trigger the default_factory; it
# returns the explicit default and leaves the dict untouched
assert dl.get("nope", "nay") == "nay"; _ledger.append(1)
assert "nope" not in dl; _ledger.append(1)

# A defaultdict converted via dict() loses its default-factory behavior
plain = dict(dl)
# But the captured entries persist
assert plain["x"] == [1, 2]; _ledger.append(1)
assert plain["y"] == [10]; _ledger.append(1)

# Nested defaultdict — counter over (group, item) pairs
counts = defaultdict(int)
records = [("a", "x"), ("a", "x"), ("a", "y"), ("b", "x")]
for g, _i in records:
    counts[g] += 1
assert counts["a"] == 3; _ledger.append(1)
assert counts["b"] == 1; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_defaultdict_ops {sum(_ledger)} asserts")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "defaultdict_int_factory_autocreates_zero"
# subject = "collections.defaultdict"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.defaultdict: defaultdict(int) auto-creates a missing key with value 0 so += accumulates; membership, len, keys/values, get, and update behave like a plain dict"""
from collections import defaultdict

d = defaultdict(int)
d["a"] += 1
d["b"] += 2
d["a"] += 10
assert d["a"] == 11 and d["b"] == 2, f"accumulated = {dict(d)!r}"
assert d["missing"] == 0, "missing key auto-creates 0"
assert sorted(d.keys()) == ["a", "b", "missing"], "missing read inserts the key"
assert sorted(d.values()) == [0, 2, 11], "values"
assert ("a" in d) and ("nope" not in d), "membership"
assert d.get("a") == 11 and d.get("never", -1) == -1, "get with default"
d.update({"z": 100})
assert d["z"] == 100, "update sets a key"

print("defaultdict_int_factory_autocreates_zero OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "defaultdict_list_factory_groups_appends"
# subject = "collections.defaultdict"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.defaultdict: defaultdict(list) auto-creates a fresh list per missing key so appends group values, and .default_factory is the list type itself"""
from collections import defaultdict

d = defaultdict(list)
d["x"].append(1)
d["x"].append(2)
d["y"].append(99)
assert d["x"] == [1, 2] and d["y"] == [99], f"grouped = {dict(d)!r}"
assert sorted(d.keys()) == ["x", "y"], "keys"
assert d.default_factory is list, "default_factory is the list type"

print("defaultdict_list_factory_groups_appends OK")

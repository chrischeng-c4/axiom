# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "abc_mapping_views_are_set_like"
# subject = "collections.abc.KeysView"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.abc.KeysView: mapping keys()/items()/values() are ABC instances and the set-like key/item views support set operations against plain sets, snapshotting at the time of the operation"""
import collections.abc as abc
from collections import UserDict

mymap = UserDict()
mymap["red"] = 5
assert isinstance(mymap.keys(), (abc.Set, abc.KeysView)), "keys is a Set/KeysView"
assert isinstance(mymap.items(), (abc.Set, abc.ItemsView)), "items is a Set/ItemsView"
assert isinstance(mymap.values(), (abc.Collection, abc.ValuesView)), "values is a ValuesView"

z = mymap.keys() | {"orange"}
assert isinstance(z, set), "keys | set -> set"
mymap["blue"] = 7  # added after the union; must not appear in z
assert sorted(z) == ["orange", "red"], f"keys union snapshot = {sorted(z)!r}"

iz = UserDict(red=5).items() | {("orange", 3)}
assert iz == {("orange", 3), ("red", 5)}, f"items union = {iz!r}"

print("abc_mapping_views_are_set_like OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "ordereddict_preserves_insertion_order"
# subject = "collections.OrderedDict"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.OrderedDict: OrderedDict iterates keys/values in insertion order regardless of key sort order"""
from collections import OrderedDict

od = OrderedDict()
od["z"] = 1
od["a"] = 2
od["m"] = 3
assert list(od.keys()) == ["z", "a", "m"], "keys in insertion order"
assert list(od.values()) == [1, 2, 3], "values in insertion order"
assert list(od.items()) == [("z", 1), ("a", 2), ("m", 3)], "items in insertion order"

print("ordereddict_preserves_insertion_order OK")

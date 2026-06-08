# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "ordereddict_move_to_end_reorders"
# subject = "collections.OrderedDict"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.OrderedDict: OrderedDict.move_to_end(key) relocates the key to the end of the iteration order"""
from collections import OrderedDict

od = OrderedDict()
od["a"] = 1
od["b"] = 2
od["c"] = 3
assert list(od.keys()) == ["a", "b", "c"], "initial order"
od.move_to_end("a")
assert list(od.keys()) == ["b", "c", "a"], "a relocated to the end"

print("ordereddict_move_to_end_reorders OK")

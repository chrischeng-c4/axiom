# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "ordereddict_equality_is_order_sensitive"
# subject = "collections.OrderedDict"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.OrderedDict: two OrderedDicts with the same items in different order compare unequal, while coercing both to plain dict makes them equal (order-agnostic)"""
from collections import OrderedDict

od1 = OrderedDict([("a", 1), ("b", 2)])
od2 = OrderedDict([("b", 2), ("a", 1)])
assert od1 != od2, "OrderedDict equality is order-sensitive"
assert dict(od1) == dict(od2), "plain dict equality is order-agnostic"

print("ordereddict_equality_is_order_sensitive OK")

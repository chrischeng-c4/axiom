# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "value_equality_inference"
# dimension = "behavior"
# case = "dict_eq_order_independent"
# subject = "dict == dict ignores insertion order"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""Dict equality compares key/value pairs by value regardless of insertion order."""
a = {"x": 1, "y": 2, "z": 3}
b = {"z": 3, "y": 2, "x": 1}
assert (a == b) is True, a == b
print("dict_eq_order_independent OK")

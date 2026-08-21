# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "value_equality_inference"
# dimension = "behavior"
# case = "list_ne_false"
# subject = "list != list is False for equal values"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""!= must be False for two distinct lists holding equal values."""
a = [1, 2, 3]
b = [1, 2, 3]
assert (a != b) is False, a != b
print("list_ne_false OK")

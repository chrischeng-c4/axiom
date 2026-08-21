# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "value_equality_inference"
# dimension = "behavior"
# case = "in_tuple_membership"
# subject = "in-operator membership tests by value over a tuple of lists"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""The in operator finds a list member of a tuple by value, not identity."""
needle = [1, 2]
container = ([0], [1, 2], [3])
assert (needle in container) is True, needle
assert ([9, 9] in container) is False
print("in_tuple_membership OK")

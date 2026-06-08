# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "value_equality_inference"
# dimension = "behavior"
# case = "nested_list_eq"
# subject = "nested list == list recurses by value"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""Equality of nested lists recurses element-by-element by value."""
a = [[1, 2], [3, [4, 5]]]
b = [[1, 2], [3, [4, 5]]]
assert (a == b) is True, a == b
c = [[1, 2], [3, [4, 6]]]
assert (a == c) is False, a == c
print("nested_list_eq OK")

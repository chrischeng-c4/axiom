# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "value_equality_inference"
# dimension = "behavior"
# case = "list_eq_locals"
# subject = "list == list compares by value"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""list == list as locals compares element values, not object identity."""
a = [1, 2]
b = [1, 2]
assert (a == b) is True, a == b
assert a is not b
print("list_eq_locals OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "builtin_numeric_inference"
# dimension = "behavior"
# case = "max_float_list_assign"
# subject = "max() over a list of floats, assigned then used"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""max() of a float list assigned to a variable must yield the correct float value."""

m = max([3.5, 1.5, 2.5])
assert m == 3.5, m
assert isinstance(m, float), type(m)
half = m / 2.0
assert half == 1.75, half
print("max_float_list_assign OK")

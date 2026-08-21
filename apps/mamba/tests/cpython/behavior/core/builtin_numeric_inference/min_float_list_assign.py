# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "builtin_numeric_inference"
# dimension = "behavior"
# case = "min_float_list_assign"
# subject = "min() over a list of floats, assigned then used"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""min() of a float list assigned to a variable must yield the correct float value."""

m = min([3.5, 1.5, 2.5])
assert m == 1.5, m
assert isinstance(m, float), type(m)
times = m * 4.0
assert times == 6.0, times
print("min_float_list_assign OK")

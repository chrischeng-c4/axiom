# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "builtin_numeric_inference"
# dimension = "behavior"
# case = "sum_float_list_assign"
# subject = "sum() over a list of floats, assigned then used"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sum() of a float list assigned to a variable must yield the float value, not leaked NaN-box bits."""

s = sum([1.5, 2.5, 3.0])
assert s == 7.0, s
assert isinstance(s, float), type(s)
# use the value after assignment (the leak shows up on assign-then-use)
doubled = s * 2.0
assert doubled == 14.0, doubled
print("sum_float_list_assign OK")

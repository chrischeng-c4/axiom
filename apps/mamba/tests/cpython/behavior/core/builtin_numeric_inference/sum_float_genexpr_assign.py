# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "builtin_numeric_inference"
# dimension = "behavior"
# case = "sum_float_genexpr_assign"
# subject = "sum() over a generator expression of floats, assigned then used"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sum() of a float generator expression assigned to a variable must yield the float value."""

s = sum(x * 1.0 for x in [1, 2, 3, 4])
assert s == 10.0, s
assert isinstance(s, float), type(s)
plus = s + 0.5
assert plus == 10.5, plus
print("sum_float_genexpr_assign OK")

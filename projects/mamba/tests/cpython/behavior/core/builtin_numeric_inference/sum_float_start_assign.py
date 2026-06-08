# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "builtin_numeric_inference"
# dimension = "behavior"
# case = "sum_float_start_assign"
# subject = "sum() with a float start argument, assigned then used"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sum([...], start=0.0) assigned to a variable must yield the float value."""

s = sum([1.5, 2.5, 4.0], 0.0)
assert s == 8.0, s
assert isinstance(s, float), type(s)
minus = s - 1.0
assert minus == 7.0, minus
print("sum_float_start_assign OK")

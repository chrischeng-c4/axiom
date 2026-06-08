# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "builtin_numeric_inference"
# dimension = "behavior"
# case = "abs_float_assign"
# subject = "abs() of a negative float, assigned then used"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""abs(-1.5) assigned to a variable must yield 1.5 as a float."""

a = abs(-1.5)
assert a == 1.5, a
assert isinstance(a, float), type(a)
plus = a + 0.5
assert plus == 2.0, plus
print("abs_float_assign OK")

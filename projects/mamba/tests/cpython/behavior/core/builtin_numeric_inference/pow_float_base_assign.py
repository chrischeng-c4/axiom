# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "builtin_numeric_inference"
# dimension = "behavior"
# case = "pow_float_base_assign"
# subject = "pow(float, int) return value, assigned then used"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""pow(2.0, 3) assigned to a variable must yield 8.0 as a float."""

p = pow(2.0, 3)
assert p == 8.0, p
assert isinstance(p, float), type(p)
half = p / 2.0
assert half == 4.0, half
print("pow_float_base_assign OK")

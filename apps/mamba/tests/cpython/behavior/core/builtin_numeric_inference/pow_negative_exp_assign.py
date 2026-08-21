# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "builtin_numeric_inference"
# dimension = "behavior"
# case = "pow_negative_exp_assign"
# subject = "pow(int, negative-int) returns float, assigned then used"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""pow(2, -1) returns the float 0.5; assigned then used must keep the float value."""

p = pow(2, -1)
assert p == 0.5, p
assert isinstance(p, float), type(p)
times = p * 8.0
assert times == 4.0, times
print("pow_negative_exp_assign OK")

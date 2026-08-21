# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "builtin_numeric_inference"
# dimension = "behavior"
# case = "round_var_args_assign"
# subject = "round(x, n) with variable args, assigned then used"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""round(x, n) where x and n are variables must yield the correct float when assigned then used."""

x = 3.14159
n = 3
r = round(x, n)
assert r == 3.142, r
assert isinstance(r, float), type(r)
scaled = r * 1000.0
assert scaled == 3142.0, scaled
print("round_var_args_assign OK")

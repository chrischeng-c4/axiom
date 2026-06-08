# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "builtin_numeric_inference"
# dimension = "behavior"
# case = "abs_int_stays_int"
# subject = "abs() of a negative int stays int, assigned then used"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""abs(-7) assigned to a variable must stay int 7."""

a = abs(-7)
assert a == 7, a
assert isinstance(a, int), type(a)
assert not isinstance(a, bool), type(a)
prod = a * 3
assert prod == 21, prod
print("abs_int_stays_int OK")

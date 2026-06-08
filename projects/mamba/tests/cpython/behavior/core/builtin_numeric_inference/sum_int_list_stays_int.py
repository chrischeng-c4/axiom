# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "builtin_numeric_inference"
# dimension = "behavior"
# case = "sum_int_list_stays_int"
# subject = "sum() over a list of ints stays int, assigned then used"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sum() of an int list assigned to a variable must stay an int with the correct value."""

s = sum([1, 2, 3])
assert s == 6, s
assert isinstance(s, int), type(s)
assert not isinstance(s, bool), type(s)
prod = s * 2
assert prod == 12, prod
print("sum_int_list_stays_int OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "mixed_numeric_inference"
# dimension = "behavior"
# case = "bool_sum_is_int"
# subject = "sum() over bools yields int total"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bool counts as int under sum(): sum([True, True]) == 2 with int type."""
n = sum([True, True])
assert n == 2, n
assert isinstance(n, int), type(n)
assert sum([True, False, True]) == 2, sum([True, False, True])
print("bool_sum_is_int OK")

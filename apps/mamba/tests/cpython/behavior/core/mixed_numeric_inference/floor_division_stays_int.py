# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "mixed_numeric_inference"
# dimension = "behavior"
# case = "floor_division_stays_int"
# subject = "int // int floor division return value and type"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""int // int floor division stays int (3 // 2 == 1, exact value and type)."""
f = 3 // 2
assert f == 1, f
assert isinstance(f, int), type(f)
assert (7 // 2) == 3, 7 // 2
print("floor_division_stays_int OK")

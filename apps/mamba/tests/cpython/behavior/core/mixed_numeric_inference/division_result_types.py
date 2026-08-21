# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "mixed_numeric_inference"
# dimension = "behavior"
# case = "division_result_types"
# subject = "true division returns float vs floor division returns int"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""Even when evenly divisible, / returns float and // returns int (4/2 is 2.0 float; 4//2 is 2 int)."""
true_div = 4 / 2
floor_div = 4 // 2
assert true_div == 2.0, true_div
assert isinstance(true_div, float), type(true_div)
assert floor_div == 2, floor_div
assert isinstance(floor_div, int), type(floor_div)
assert true_div == floor_div, (true_div, floor_div)
print("division_result_types OK")

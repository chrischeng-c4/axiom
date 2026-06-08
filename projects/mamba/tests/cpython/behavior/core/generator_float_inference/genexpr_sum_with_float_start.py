# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "generator_float_inference"
# dimension = "behavior"
# case = "genexpr_sum_with_float_start"
# subject = "sum() of an int genexpr with a float start value returns a float"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sum() over a generator expression with a float start value must return the correct float total."""

total = sum((i for i in range(5)), 0.5)
assert isinstance(total, float), type(total)
assert total == 10.5, total
print("genexpr_sum_with_float_start OK")

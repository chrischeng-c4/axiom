# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "generator_float_inference"
# dimension = "behavior"
# case = "genexpr_sum_floats"
# subject = "generator expression of floats reduced by sum()"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A generator expression producing floats, summed, must yield the correct float total."""

data = [1, 2, 3, 4]
total = sum(x * 1.5 for x in data)
assert isinstance(total, float), type(total)
assert total == 15.0, total
print("genexpr_sum_floats OK")

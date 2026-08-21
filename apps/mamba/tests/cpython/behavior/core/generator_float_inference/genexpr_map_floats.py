# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "generator_float_inference"
# dimension = "behavior"
# case = "genexpr_map_floats"
# subject = "generator expression of floats consumed via map()"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A float generator expression consumed by map() must produce correct transformed floats."""

source = (i / 4.0 for i in range(4))
doubled = list(map(lambda v: v * 2.0, source))
assert doubled == [0.0, 0.5, 1.0, 1.5], doubled
assert all(isinstance(x, float) for x in doubled), doubled
print("genexpr_map_floats OK")

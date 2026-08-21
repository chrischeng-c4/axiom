# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "container_float_roundtrip"
# dimension = "behavior"
# case = "list_of_computed_floats_sum"
# subject = "sum over a list of computed floats"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""Computed floats stored in a list must sum to the correct float total."""
xs = []
for i in range(4):
    xs.append(i * 0.5)
assert xs == [0.0, 0.5, 1.0, 1.5], xs
total = sum(xs)
assert total == 3.0, total
assert isinstance(total, float), type(total)
print("list_of_computed_floats_sum OK")

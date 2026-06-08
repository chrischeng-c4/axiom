# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "mixed_numeric_inference"
# dimension = "behavior"
# case = "int_times_float"
# subject = "int * float product value and type"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""int * float yields the correct float product (2 * 1.5 == 3.0), not NaN-box bits."""
p = 2 * 1.5
assert p == 3.0, p
assert isinstance(p, float), type(p)
assert (3 * 0.25) == 0.75, 3 * 0.25
print("int_times_float OK")

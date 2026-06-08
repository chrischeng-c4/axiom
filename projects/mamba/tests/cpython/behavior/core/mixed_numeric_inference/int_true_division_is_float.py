# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "mixed_numeric_inference"
# dimension = "behavior"
# case = "int_true_division_is_float"
# subject = "int / int true division return value and type"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""int / int true division yields a float, never integer floor (1/2 == 0.5)."""
q = 1 / 2
assert q == 0.5, q
assert isinstance(q, float), type(q)
assert (7 / 2) == 3.5, 7 / 2
print("int_true_division_is_float OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "mixed_numeric_inference"
# dimension = "behavior"
# case = "int_mod_float"
# subject = "int % float remainder value and type"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""int % float promotes to a float remainder (5 % 2.0 == 1.0), not bit garbage."""
m = 5 % 2.0
assert m == 1.0, m
assert isinstance(m, float), type(m)
assert (5.5 % 2) == 1.5, 5.5 % 2
print("int_mod_float OK")

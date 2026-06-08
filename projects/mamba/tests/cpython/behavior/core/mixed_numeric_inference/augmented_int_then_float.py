# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "mixed_numeric_inference"
# dimension = "behavior"
# case = "augmented_int_then_float"
# subject = "augmented assignment int += float promotes value and type"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""x bound to int then x += 0.5 rebinds to the correct float (0.5), not bit garbage."""
x = 0
x += 0.5
assert x == 0.5, x
assert isinstance(x, float), type(x)
x += 1
assert x == 1.5, x
assert isinstance(x, float), type(x)
print("augmented_int_then_float OK")

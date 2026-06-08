# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "mixed_numeric_inference"
# dimension = "behavior"
# case = "int_pow_half_is_sqrt"
# subject = "int ** float fractional exponent yields float root"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""int ** 0.5 returns the float square root (4 ** 0.5 == 2.0; 2 ** 0.5 ~ 1.4142)."""
root4 = 4 ** 0.5
assert root4 == 2.0, root4
assert isinstance(root4, float), type(root4)
root2 = 2 ** 0.5
assert isinstance(root2, float), type(root2)
assert abs(root2 - 1.4142135623730951) < 1e-12, root2
assert abs(root2 * root2 - 2.0) < 1e-12, root2 * root2
print("int_pow_half_is_sqrt OK")

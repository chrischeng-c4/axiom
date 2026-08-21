# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "surface"
# case = "sqrt_is_callable"
# subject = "math.sqrt"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.sqrt: sqrt_is_callable (surface)."""
import math

assert callable(math.sqrt)
print("sqrt_is_callable OK")

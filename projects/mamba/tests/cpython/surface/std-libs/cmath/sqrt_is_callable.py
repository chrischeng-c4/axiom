# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "surface"
# case = "sqrt_is_callable"
# subject = "cmath.sqrt"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cmath.sqrt: sqrt_is_callable (surface)."""
import cmath

assert callable(cmath.sqrt)
print("sqrt_is_callable OK")

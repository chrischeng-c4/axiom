# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "surface"
# case = "sin_is_callable"
# subject = "cmath.sin"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cmath.sin: sin_is_callable (surface)."""
import cmath

assert callable(cmath.sin)
print("sin_is_callable OK")

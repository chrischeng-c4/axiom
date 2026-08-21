# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "surface"
# case = "tan_is_callable"
# subject = "cmath.tan"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cmath.tan: tan_is_callable (surface)."""
import cmath

assert callable(cmath.tan)
print("tan_is_callable OK")

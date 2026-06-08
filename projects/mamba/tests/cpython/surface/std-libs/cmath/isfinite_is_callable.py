# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "surface"
# case = "isfinite_is_callable"
# subject = "cmath.isfinite"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cmath.isfinite: isfinite_is_callable (surface)."""
import cmath

assert callable(cmath.isfinite)
print("isfinite_is_callable OK")

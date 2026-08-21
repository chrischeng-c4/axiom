# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "surface"
# case = "isnan_is_callable"
# subject = "cmath.isnan"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cmath.isnan: isnan_is_callable (surface)."""
import cmath

assert callable(cmath.isnan)
print("isnan_is_callable OK")

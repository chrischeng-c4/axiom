# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "surface"
# case = "cos_is_callable"
# subject = "cmath.cos"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cmath.cos: cos_is_callable (surface)."""
import cmath

assert callable(cmath.cos)
print("cos_is_callable OK")

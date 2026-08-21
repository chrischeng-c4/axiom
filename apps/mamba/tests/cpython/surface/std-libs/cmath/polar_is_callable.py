# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "surface"
# case = "polar_is_callable"
# subject = "cmath.polar"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cmath.polar: polar_is_callable (surface)."""
import cmath

assert callable(cmath.polar)
print("polar_is_callable OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "surface"
# case = "isclose_is_callable"
# subject = "cmath.isclose"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cmath.isclose: isclose_is_callable (surface)."""
import cmath

assert callable(cmath.isclose)
print("isclose_is_callable OK")

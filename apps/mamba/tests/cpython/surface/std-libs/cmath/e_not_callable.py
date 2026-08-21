# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "surface"
# case = "e_not_callable"
# subject = "cmath.e"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cmath.e: e_not_callable (surface)."""
import cmath

assert not callable(cmath.e)
print("e_not_callable OK")

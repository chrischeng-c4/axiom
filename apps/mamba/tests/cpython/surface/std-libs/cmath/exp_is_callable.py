# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "surface"
# case = "exp_is_callable"
# subject = "cmath.exp"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cmath.exp: exp_is_callable (surface)."""
import cmath

assert callable(cmath.exp)
print("exp_is_callable OK")

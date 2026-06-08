# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "behavior"
# case = "sqrt_positive_real_result"
# subject = "cmath.sqrt"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cmath.sqrt: sqrt(4) returns a complex equal to 2 (zero imaginary part)"""
import cmath

_sq = cmath.sqrt(4)
assert isinstance(_sq, complex), f"sqrt(4) type = {type(_sq)!r}"
assert abs(_sq - 2) < 1e-15, f"sqrt(4) = {_sq!r}"
print("sqrt_positive_real_result OK")

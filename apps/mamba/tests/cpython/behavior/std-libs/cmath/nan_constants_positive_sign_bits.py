# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "behavior"
# case = "nan_constants_positive_sign_bits"
# subject = "cmath.nan"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cmath.nan: every component of the cmath.nan / cmath.nanj constants has a positive sign bit (copysign(1, c) == 1)"""
import cmath
import math

assert math.copysign(1.0, cmath.nan.real) == 1.0, "nan.real sign +"
assert math.copysign(1.0, cmath.nan.imag) == 1.0, "nan.imag sign +"
assert math.copysign(1.0, cmath.nanj.real) == 1.0, "nanj.real sign +"
assert math.copysign(1.0, cmath.nanj.imag) == 1.0, "nanj.imag sign +"
print("nan_constants_positive_sign_bits OK")

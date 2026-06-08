# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "behavior"
# case = "nan_carries_one_nan_component"
# subject = "cmath.nan"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cmath.nan: cmath.nan carries NaN in the real component and a clean +0 imaginary; nanj is the imaginary-axis mirror"""
import cmath
import math

assert math.isnan(cmath.nan.real), "nan.real is NaN"
assert cmath.nan.imag == 0.0, f"nan.imag = {cmath.nan.imag!r}"
assert cmath.nanj.real == 0.0, f"nanj.real = {cmath.nanj.real!r}"
assert math.isnan(cmath.nanj.imag), "nanj.imag is NaN"
print("nan_carries_one_nan_component OK")

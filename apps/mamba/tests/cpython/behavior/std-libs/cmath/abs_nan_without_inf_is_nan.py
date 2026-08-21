# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "behavior"
# case = "abs_nan_without_inf_is_nan"
# subject = "abs"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""abs: abs() is NaN when a component is NaN and no component is infinite"""
import math

NAN = float("nan")

assert math.isnan(abs(complex(NAN, 2.3))), "abs(nan+2.3j) is nan"
assert math.isnan(abs(complex(-2.3, NAN))), "abs(-2.3+nanj) is nan"
assert math.isnan(abs(complex(NAN, NAN))), "abs(nan+nanj) is nan"
print("abs_nan_without_inf_is_nan OK")

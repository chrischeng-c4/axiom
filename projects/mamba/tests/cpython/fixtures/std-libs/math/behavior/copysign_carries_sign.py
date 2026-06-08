# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "behavior"
# case = "copysign_carries_sign"
# subject = "math.copysign"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.copysign: copysign carries the sign bit of the second arg onto the magnitude of the first: copysign(3, -1)==-3.0, copysign(-5, 1)==5.0, and copysign(1, nan) uses NaN's positive sign -> 1.0"""
import math

assert math.copysign(3.0, -1.0) == -3.0, f"copysign(3,-1) = {math.copysign(3.0, -1.0)!r}"
assert math.copysign(-5.0, 1.0) == 5.0, f"copysign(-5,1) = {math.copysign(-5.0, 1.0)!r}"
assert math.copysign(1.0, math.nan) == 1.0, f"copysign(1,nan) = {math.copysign(1.0, math.nan)!r}"

print("copysign_carries_sign OK")

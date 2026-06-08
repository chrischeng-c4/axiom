# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "behavior"
# case = "inverse_trig"
# subject = "math.atan2"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.atan2: inverse-trig round trips: asin(0)==0, acos(1)==0, atan(0)==0, and atan2(1, 1)==pi/4 within 1e-10"""
import math

_eps = 1e-10
assert abs(math.asin(0) - 0.0) < _eps, f"asin(0) = {math.asin(0)!r}"
assert abs(math.acos(1) - 0.0) < _eps, f"acos(1) = {math.acos(1)!r}"
assert abs(math.atan(0) - 0.0) < _eps, f"atan(0) = {math.atan(0)!r}"
assert abs(math.atan2(1, 1) - math.pi / 4) < _eps, f"atan2(1,1) = {math.atan2(1, 1)!r}"

print("inverse_trig OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "behavior"
# case = "isqrt_floor_root"
# subject = "math.isqrt"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.isqrt: math.isqrt returns the integer floor of the square root: isqrt(0)==0, isqrt(99)==9, isqrt(100)==10, isqrt(101)==10, isqrt(10**12)==1000000"""
import math

assert math.isqrt(0) == 0, f"isqrt(0) = {math.isqrt(0)!r}"
assert math.isqrt(99) == 9, f"isqrt(99) = {math.isqrt(99)!r}"
assert math.isqrt(100) == 10, f"isqrt(100) = {math.isqrt(100)!r}"
assert math.isqrt(101) == 10, f"isqrt(101) = {math.isqrt(101)!r}"
assert math.isqrt(10**12) == 1000000, f"isqrt(10**12) = {math.isqrt(10**12)!r}"

print("isqrt_floor_root OK")

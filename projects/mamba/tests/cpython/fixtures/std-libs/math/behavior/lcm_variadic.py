# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "behavior"
# case = "lcm_variadic"
# subject = "math.lcm"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.lcm: math.lcm is variadic: lcm()==1, lcm(-7)==7 (abs), lcm(4, 6)==12, lcm(4, 6, 9)==36, lcm(2, 3, 5, 7)==210, and any zero argument short-circuits to 0"""
import math

assert math.lcm() == 1, f"lcm() = {math.lcm()!r}"
assert math.lcm(-7) == 7, f"lcm(-7) = {math.lcm(-7)!r}"
assert math.lcm(4, 6) == 12, f"lcm(4,6) = {math.lcm(4, 6)!r}"
assert math.lcm(4, 6, 9) == 36, f"lcm(4,6,9) = {math.lcm(4, 6, 9)!r}"
assert math.lcm(2, 3, 5, 7) == 210, f"lcm(2,3,5,7) = {math.lcm(2, 3, 5, 7)!r}"
assert math.lcm(4, 0, 9) == 0, f"lcm(4,0,9) = {math.lcm(4, 0, 9)!r}"

print("lcm_variadic OK")

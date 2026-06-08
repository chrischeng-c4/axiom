# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "behavior"
# case = "remainder_round_half_even"
# subject = "math.remainder"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.remainder: IEEE-754 remainder rounds the quotient to nearest-even: remainder(7, 3)==1.0, remainder(7.5, 5)==-2.5, remainder(-7, 3)==-1.0, remainder(10, 4)==2.0"""
import math

assert math.remainder(7, 3) == 1.0, f"remainder(7,3) = {math.remainder(7, 3)!r}"
assert math.remainder(7.5, 5) == -2.5, f"remainder(7.5,5) = {math.remainder(7.5, 5)!r}"
assert math.remainder(-7, 3) == -1.0, f"remainder(-7,3) = {math.remainder(-7, 3)!r}"
assert math.remainder(10, 4) == 2.0, f"remainder(10,4) = {math.remainder(10, 4)!r}"

print("remainder_round_half_even OK")

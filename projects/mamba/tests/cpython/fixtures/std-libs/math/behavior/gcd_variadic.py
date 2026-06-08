# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "behavior"
# case = "gcd_variadic"
# subject = "math.gcd"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.gcd: math.gcd is variadic (CPython 3.9+): gcd()==0, gcd(7)==7, gcd(-12)==12 (abs), gcd(12, 18)==6, gcd(48, 36, 60, 84)==12"""
import math

assert math.gcd() == 0, f"gcd() = {math.gcd()!r}"
assert math.gcd(7) == 7, f"gcd(7) = {math.gcd(7)!r}"
assert math.gcd(-12) == 12, f"gcd(-12) = {math.gcd(-12)!r}"
assert math.gcd(12, 18) == 6, f"gcd(12,18) = {math.gcd(12, 18)!r}"
assert math.gcd(48, 36, 60, 84) == 12, f"gcd(48,36,60,84) = {math.gcd(48, 36, 60, 84)!r}"

print("gcd_variadic OK")

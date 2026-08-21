# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "behavior"
# case = "ldexp_power_of_two"
# subject = "math.ldexp"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.ldexp: math.ldexp(x, i) scales x by 2**i exactly: ldexp(1.5, 3)==12.0, ldexp(1.0, -3)==0.125, ldexp(0.0, 100)==0.0"""
import math

assert math.ldexp(1.5, 3) == 12.0, f"ldexp(1.5,3) = {math.ldexp(1.5, 3)!r}"
assert math.ldexp(1.0, -3) == 0.125, f"ldexp(1.0,-3) = {math.ldexp(1.0, -3)!r}"
assert math.ldexp(0.0, 100) == 0.0, f"ldexp(0.0,100) = {math.ldexp(0.0, 100)!r}"

print("ldexp_power_of_two OK")

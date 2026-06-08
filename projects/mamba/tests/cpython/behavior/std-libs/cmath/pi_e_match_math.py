# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "behavior"
# case = "pi_e_match_math"
# subject = "cmath.pi"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cmath.pi: cmath.pi and cmath.e agree with the math module values to float tolerance"""
import cmath
import math

assert abs(cmath.pi - math.pi) < 1e-15, f"cmath.pi = {cmath.pi!r}"
assert abs(cmath.e - math.e) < 1e-15, f"cmath.e = {cmath.e!r}"
print("pi_e_match_math OK")

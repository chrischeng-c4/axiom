# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "behavior"
# case = "constants_values"
# subject = "math.pi"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.pi: the named constants carry their canonical IEEE-754 values: pi==3.141592653589793, e==2.718281828459045, tau==2*pi; inf is infinite and nan != nan"""
import math

assert math.pi == 3.141592653589793, f"pi = {math.pi!r}"
assert math.e == 2.718281828459045, f"e = {math.e!r}"
assert abs(math.tau - 2 * math.pi) < 1e-10, f"tau = {math.tau!r}"
assert math.isinf(math.inf), "inf is infinite"
assert math.nan != math.nan, "nan != nan"

print("constants_values OK")

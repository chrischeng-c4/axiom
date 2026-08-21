# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "behavior"
# case = "exp_underflow_to_zero"
# subject = "math.exp"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.exp: exp(0)==1.0, exp(1) is e, and exp of a huge negative argument underflows silently to 0.0 (no exception)"""
import math

assert math.exp(0) == 1.0, f"exp(0) = {math.exp(0)!r}"
assert abs(math.exp(1) - math.e) < 1e-10, f"exp(1) = {math.exp(1)!r}"
assert math.exp(-1000000000) == 0.0, f"exp(-1e9) = {math.exp(-1000000000)!r}"

print("exp_underflow_to_zero OK")

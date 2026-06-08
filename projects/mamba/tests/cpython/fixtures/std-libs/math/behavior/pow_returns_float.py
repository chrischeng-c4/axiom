# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "behavior"
# case = "pow_returns_float"
# subject = "math.pow"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.pow: math.pow always returns a float: pow(2, 10)==1024.0, pow(5, 0)==1.0, pow(2, -1)==0.5, pow(2, 0.5) is sqrt(2)"""
import math

assert math.pow(2, 10) == 1024.0, f"pow(2,10) = {math.pow(2, 10)!r}"
assert isinstance(math.pow(2, 10), float), "pow returns float"
assert math.pow(5, 0) == 1.0, f"pow(5,0) = {math.pow(5, 0)!r}"
assert math.pow(2, -1) == 0.5, f"pow(2,-1) = {math.pow(2, -1)!r}"
assert abs(math.pow(2, 0.5) - math.sqrt(2)) < 1e-10, f"pow(2,0.5) = {math.pow(2, 0.5)!r}"

print("pow_returns_float OK")

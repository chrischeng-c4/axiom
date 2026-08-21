# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "behavior"
# case = "sqrt_returns_float"
# subject = "math.sqrt"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.sqrt: math.sqrt always returns a float (sqrt(4) is 2.0, not int 2) and sqrt(2)==1.4142135623730951; sqrt(0)==0.0, sqrt(1)==1.0"""
import math

assert isinstance(math.sqrt(4), float), f"sqrt type = {type(math.sqrt(4))!r}"
assert math.sqrt(4) == 2.0, f"sqrt(4) = {math.sqrt(4)!r}"
assert math.sqrt(2) == 1.4142135623730951, f"sqrt(2) = {math.sqrt(2)!r}"
assert math.sqrt(0) == 0.0, f"sqrt(0) = {math.sqrt(0)!r}"
assert math.sqrt(1) == 1.0, f"sqrt(1) = {math.sqrt(1)!r}"

print("sqrt_returns_float OK")

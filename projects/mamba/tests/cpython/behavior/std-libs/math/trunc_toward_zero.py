# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "behavior"
# case = "trunc_toward_zero"
# subject = "math.trunc"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.trunc: math.trunc rounds toward zero and returns int: trunc(3.7)==3, trunc(-3.7)==-3, trunc(0.5)==0"""
import math

assert math.trunc(3.7) == 3, f"trunc(3.7) = {math.trunc(3.7)!r}"
assert math.trunc(-3.7) == -3, f"trunc(-3.7) = {math.trunc(-3.7)!r}"
assert math.trunc(0.5) == 0, f"trunc(0.5) = {math.trunc(0.5)!r}"
assert isinstance(math.trunc(3.7), int), f"trunc type = {type(math.trunc(3.7))!r}"

print("trunc_toward_zero OK")

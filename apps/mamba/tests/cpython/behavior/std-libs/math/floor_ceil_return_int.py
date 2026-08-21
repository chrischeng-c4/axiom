# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "behavior"
# case = "floor_ceil_return_int"
# subject = "math.floor"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.floor: in Python 3.12 math.floor and math.ceil return int (not float); floor rounds toward -inf, ceil toward +inf, across positive/negative/integral inputs"""
import math

assert isinstance(math.floor(3.7), int), f"floor type = {type(math.floor(3.7))!r}"
assert isinstance(math.ceil(3.2), int), f"ceil type = {type(math.ceil(3.2))!r}"
assert math.floor(3.7) == 3, f"floor(3.7) = {math.floor(3.7)!r}"
assert math.floor(-3.2) == -4, f"floor(-3.2) = {math.floor(-3.2)!r}"
assert math.floor(5.0) == 5, f"floor(5.0) = {math.floor(5.0)!r}"
assert math.ceil(3.2) == 4, f"ceil(3.2) = {math.ceil(3.2)!r}"
assert math.ceil(-3.7) == -3, f"ceil(-3.7) = {math.ceil(-3.7)!r}"
assert math.ceil(5.0) == 5, f"ceil(5.0) = {math.ceil(5.0)!r}"

print("floor_ceil_return_int OK")

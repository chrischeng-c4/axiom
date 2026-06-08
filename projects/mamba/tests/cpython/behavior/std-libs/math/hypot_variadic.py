# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "behavior"
# case = "hypot_variadic"
# subject = "math.hypot"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.hypot: math.hypot is variadic: hypot()==0.0, hypot(-5)==5.0, hypot(3, 4)==5.0 (2-arg), hypot(3, 4, 12)==13.0 (N-arg Euclidean norm)"""
import math

assert math.hypot() == 0.0, f"hypot() = {math.hypot()!r}"
assert math.hypot(-5) == 5.0, f"hypot(-5) = {math.hypot(-5)!r}"
assert math.hypot(5) == 5.0, f"hypot(5) = {math.hypot(5)!r}"
assert math.hypot(3, 4) == 5.0, f"hypot(3,4) = {math.hypot(3, 4)!r}"
assert math.hypot(3, 4, 12) == 13.0, f"hypot(3,4,12) = {math.hypot(3, 4, 12)!r}"

print("hypot_variadic OK")

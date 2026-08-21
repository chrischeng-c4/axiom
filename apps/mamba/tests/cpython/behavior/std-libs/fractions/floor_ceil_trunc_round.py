# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fractions"
# dimension = "behavior"
# case = "floor_ceil_trunc_round"
# subject = "fractions.Fraction"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fractions.py"
# status = "filled"
# ///
"""fractions.Fraction: math.floor/ceil/trunc and round operate on a Fraction: floor(7/3)=2, ceil(7/3)=3, trunc(-7/3)=-2, round(5/2)=2 (banker's rounding)"""
import math
from fractions import Fraction

assert math.floor(Fraction(7, 3)) == 2, f"floor(7/3) = {math.floor(Fraction(7, 3))!r}"
assert math.ceil(Fraction(7, 3)) == 3, f"ceil(7/3) = {math.ceil(Fraction(7, 3))!r}"
assert math.trunc(Fraction(-7, 3)) == -2, f"trunc(-7/3) = {math.trunc(Fraction(-7, 3))!r}"
# round uses banker's rounding (round-half-to-even).
assert round(Fraction(5, 2)) == 2, f"round(5/2) = {round(Fraction(5, 2))!r}"

print("floor_ceil_trunc_round OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fractions"
# dimension = "behavior"
# case = "denominator_always_positive"
# subject = "fractions.Fraction"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fractions.py"
# status = "filled"
# ///
"""fractions.Fraction: the denominator is always positive: 1/-2 normalizes the sign onto the numerator (-1/2)"""
from fractions import Fraction

assert Fraction(1, -2).numerator == -1, "sign moves to numerator"
assert Fraction(1, -2).denominator == 2, "denominator is positive"
assert Fraction(-3, -4).numerator == 3, "double negative is positive"
assert Fraction(-3, -4).denominator == 4, "double negative denom positive"

print("denominator_always_positive OK")

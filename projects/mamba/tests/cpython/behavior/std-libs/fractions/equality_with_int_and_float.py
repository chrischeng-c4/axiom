# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fractions"
# dimension = "behavior"
# case = "equality_with_int_and_float"
# subject = "fractions.Fraction"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fractions.py"
# status = "filled"
# ///
"""fractions.Fraction: a Fraction compares equal across numeric types: Fraction(6,3) == 2 and == 2.0, and Fraction(1,2) == 0.5"""
from fractions import Fraction

assert Fraction(6, 3) == 2, "Fraction(6, 3) == int 2"
assert Fraction(6, 3) == 2.0, "Fraction(6, 3) == float 2.0"
assert Fraction(1, 2) == 0.5, "Fraction(1, 2) == 0.5"
assert Fraction(1, 2) == Fraction(2, 4), "equality after reduction"

print("equality_with_int_and_float OK")

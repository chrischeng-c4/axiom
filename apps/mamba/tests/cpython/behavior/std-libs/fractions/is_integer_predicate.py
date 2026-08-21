# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fractions"
# dimension = "behavior"
# case = "is_integer_predicate"
# subject = "fractions.Fraction.is_integer"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fractions.py"
# status = "filled"
# ///
"""fractions.Fraction.is_integer: is_integer() is True exactly when the reduced denominator is 1: True for 1/1, 2/2, -2/2, 4/-2 and False for 1/2, -1/2, 3/4"""
from fractions import Fraction

assert Fraction(1, 1).is_integer(), "1/1 is integer"
assert Fraction(2, 2).is_integer(), "2/2 reduces to integer"
assert Fraction(-2, 2).is_integer(), "-2/2 is integer"
assert Fraction(4, -2).is_integer(), "4/-2 is integer"
assert not Fraction(1, 2).is_integer(), "1/2 is not integer"
assert not Fraction(-1, 2).is_integer(), "-1/2 is not integer"
assert not Fraction(3, 4).is_integer(), "3/4 is not integer"

print("is_integer_predicate OK")

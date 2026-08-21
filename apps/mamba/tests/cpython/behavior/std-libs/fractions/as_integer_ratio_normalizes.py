# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fractions"
# dimension = "behavior"
# case = "as_integer_ratio_normalizes"
# subject = "fractions.Fraction.as_integer_ratio"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fractions.py"
# status = "filled"
# ///
"""fractions.Fraction.as_integer_ratio: as_integer_ratio returns a reduced (numerator, denominator) tuple with the sign on the numerator: 4/6 -> (2,3), 4/-6 -> (-2,3), 0/6 -> (0,1), Fraction(7) -> (7,1)"""
from fractions import Fraction

assert Fraction(4, 6).as_integer_ratio() == (2, 3), "4/6 ratio reduced"
assert Fraction(-4, 6).as_integer_ratio() == (-2, 3), "-4/6 ratio"
assert Fraction(4, -6).as_integer_ratio() == (-2, 3), "sign moves to numerator"
assert Fraction(0, 6).as_integer_ratio() == (0, 1), "zero ratio"
assert Fraction(7).as_integer_ratio() == (7, 1), "integer ratio"

print("as_integer_ratio_normalizes OK")

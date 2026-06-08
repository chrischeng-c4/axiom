# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fractions"
# dimension = "behavior"
# case = "numbers_abc_membership"
# subject = "fractions.Fraction"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fractions.py"
# status = "filled"
# ///
"""fractions.Fraction: a Fraction registers as a numbers.Rational (and therefore Number) but not numbers.Integral"""
import numbers
from fractions import Fraction

assert isinstance(Fraction(1, 2), numbers.Rational), "Fraction is Rational"
assert isinstance(Fraction(1, 2), numbers.Number), "Fraction is Number"
assert not isinstance(Fraction(1, 2), numbers.Integral), "Fraction is not Integral"

print("numbers_abc_membership OK")

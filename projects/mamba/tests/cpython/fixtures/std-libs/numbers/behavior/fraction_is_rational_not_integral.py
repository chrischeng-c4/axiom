# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "numbers"
# dimension = "behavior"
# case = "fraction_is_rational_not_integral"
# subject = "numbers.Rational"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fractions.py"
# status = "filled"
# ///
"""numbers.Rational: fractions.Fraction is a Rational and a Real but not an Integral"""
import numbers
from fractions import Fraction

half = Fraction(1, 2)

# Fraction is registered exactly at the Rational rung.
assert isinstance(half, numbers.Rational), "Fraction is Rational"
assert isinstance(half, numbers.Real), "Rational is also Real"
assert isinstance(half, numbers.Complex), "Rational is also Complex"
assert isinstance(half, numbers.Number), "Rational is also Number"

# But a fraction is not an integer, so it stops above the Integral rung.
assert not isinstance(half, numbers.Integral), "Fraction is NOT Integral"

print("fraction_is_rational_not_integral OK")

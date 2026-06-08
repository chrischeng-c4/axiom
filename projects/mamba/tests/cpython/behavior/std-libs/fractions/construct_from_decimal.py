# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fractions"
# dimension = "behavior"
# case = "construct_from_decimal"
# subject = "fractions.Fraction"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fractions.py"
# status = "filled"
# ///
"""fractions.Fraction: Fraction(Decimal('1.5')) constructs the exact rational 3/2 from a decimal.Decimal"""
from decimal import Decimal
from fractions import Fraction

assert Fraction(Decimal("1.5")) == Fraction(3, 2), "Decimal 1.5 -> 3/2"
assert Fraction(Decimal("0.25")) == Fraction(1, 4), "Decimal 0.25 -> 1/4"

print("construct_from_decimal OK")

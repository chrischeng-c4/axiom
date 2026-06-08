# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "behavior"
# case = "cross_type_vs_fraction"
# subject = "decimal.Decimal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.Decimal: Decimal vs Fraction is exact rational comparison: Decimal('0.1') == Fraction(1,10), Decimal(0) < Fraction(1,7), Decimal('inf') > a huge Fraction"""
from decimal import Decimal
from fractions import Fraction

# Decimal vs Fraction: exact rational comparison.
assert Decimal("0.1") == Fraction(1, 10), "Decimal == Fraction"
assert Decimal(0) < Fraction(1, 7), "Decimal < Fraction"
assert Decimal("inf") > Fraction(10 ** 20, 3), "inf > huge Fraction"

print("cross_type_vs_fraction OK")

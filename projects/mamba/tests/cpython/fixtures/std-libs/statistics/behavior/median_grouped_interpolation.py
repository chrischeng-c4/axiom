# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "median_grouped_interpolation"
# subject = "statistics.median_grouped"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
"""statistics.median_grouped: median_grouped treats each value as a unit-interval midpoint and interpolates within the median class (Fraction and Decimal inputs), and a run of one repeated value collapses to that value as a float"""
from decimal import Decimal
from fractions import Fraction
from statistics import median_grouped

# Each value is the midpoint of a unit interval; interpolate within the
# median class.
assert median_grouped([Fraction(5, 4), Fraction(9, 4), Fraction(13, 4),
                       Fraction(13, 4), Fraction(17, 4)]) == 3.0
assert median_grouped([Decimal("5.5"), Decimal("5.5"), Decimal("6.5"),
                       Decimal("6.5"), Decimal("7.5"), Decimal("8.5")]) == 6.5
# A run of one repeated value collapses to that value, as a float.
for x in (5.3, 68, Fraction(29, 101), Decimal("32.9714")):
    assert median_grouped([x] * 5) == float(x), repr(x)

print("median_grouped_interpolation OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fractions"
# dimension = "behavior"
# case = "exact_decimal_arithmetic"
# subject = "fractions.Fraction"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fractions.py"
# status = "filled"
# ///
"""fractions.Fraction: exact arithmetic with no float error: 1/10 + 2/10 == 3/10 exactly, unlike float 0.1 + 0.2 != 0.3"""
from fractions import Fraction

assert Fraction(1, 10) + Fraction(2, 10) == Fraction(3, 10), "0.1 + 0.2 == 0.3 exact"
# Contrast: binary float arithmetic is not exact here.
assert 0.1 + 0.2 != 0.3, "float 0.1 + 0.2 != 0.3 (contrast)"
assert Fraction(1, 3) + Fraction(1, 6) == Fraction(1, 2), "1/3 + 1/6 == 1/2"

print("exact_decimal_arithmetic OK")

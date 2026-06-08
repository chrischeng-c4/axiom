# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fractions"
# dimension = "behavior"
# case = "mixed_arithmetic_with_int"
# subject = "fractions.Fraction"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fractions.py"
# status = "filled"
# ///
"""fractions.Fraction: arithmetic mixes with int and reduces to a plain int when whole: Fraction(1,2) + 1 == 3/2 and Fraction(3,4) * 4 == 3"""
from fractions import Fraction

assert Fraction(1, 2) + 1 == Fraction(3, 2), "Fraction + int"
assert Fraction(3, 4) * 4 == 3, "Fraction * int reduces to whole int value"
assert 1 - Fraction(1, 4) == Fraction(3, 4), "int - Fraction (reflected)"

print("mixed_arithmetic_with_int OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fractions"
# dimension = "behavior"
# case = "reduces_to_lowest_terms"
# subject = "fractions.Fraction"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fractions.py"
# status = "filled"
# ///
"""fractions.Fraction: Fraction always reduces to lowest terms: 12/8 -> 3/2 and -4/6 -> -2/3"""
from fractions import Fraction

assert Fraction(12, 8).numerator == 3, f"12/8 num = {Fraction(12, 8).numerator!r}"
assert Fraction(12, 8).denominator == 2, f"12/8 den = {Fraction(12, 8).denominator!r}"
assert Fraction(-4, 6).numerator == -2, f"-4/6 num = {Fraction(-4, 6).numerator!r}"
assert Fraction(-4, 6).denominator == 3, f"-4/6 den = {Fraction(-4, 6).denominator!r}"

print("reduces_to_lowest_terms OK")

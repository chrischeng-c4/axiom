# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fractions"
# dimension = "behavior"
# case = "construct_from_string"
# subject = "fractions.Fraction"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fractions.py"
# status = "filled"
# ///
"""fractions.Fraction: Fraction parses fraction and decimal strings: '3/4' -> 3/4, '0.5' -> 1/2, '-3/4' -> -3/4, with surrounding whitespace stripped"""
from fractions import Fraction

assert Fraction("3/4") == Fraction(3, 4), "from fraction string"
assert Fraction("0.5") == Fraction(1, 2), "from decimal string"
assert Fraction("-3/4") == Fraction(-3, 4), "negative fraction string"
assert Fraction("  3/4  ") == Fraction(3, 4), "surrounding whitespace stripped"

print("construct_from_string OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fractions"
# dimension = "behavior"
# case = "ordering_with_int_and_float"
# subject = "fractions.Fraction"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fractions.py"
# status = "filled"
# ///
"""fractions.Fraction: Fraction ordering mixes with numeric types: 1/3 < 0.5, 2/3 > 0.5, and equality after reduction (1/2 == 2/4) with 1/3 < 1/2 < 3/4"""
from fractions import Fraction

assert Fraction(1, 3) < 0.5, "1/3 < 0.5"
assert Fraction(2, 3) > 0.5, "2/3 > 0.5"
assert Fraction(1, 3) < Fraction(1, 2) < Fraction(3, 4), "1/3 < 1/2 < 3/4"
assert Fraction(1, 2) == Fraction(2, 4), "1/2 == 2/4 after reduction"

print("ordering_with_int_and_float OK")

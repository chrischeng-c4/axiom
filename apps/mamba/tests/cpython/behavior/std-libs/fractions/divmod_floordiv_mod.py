# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fractions"
# dimension = "behavior"
# case = "divmod_floordiv_mod"
# subject = "fractions.Fraction"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fractions.py"
# status = "filled"
# ///
"""fractions.Fraction: floor division, modulo and divmod stay exact: 7/2 // 2 == 1, 7/2 % 2 == 3/2, divmod(7/2, 2) == (1, 3/2)"""
from fractions import Fraction

assert Fraction(7, 2) // 2 == 1, "7/2 // 2 == 1"
assert Fraction(7, 2) % 2 == Fraction(3, 2), "7/2 % 2 == 3/2"
assert divmod(Fraction(7, 2), 2) == (1, Fraction(3, 2)), "divmod(7/2, 2)"

print("divmod_floordiv_mod OK")

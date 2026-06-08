# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fractions"
# dimension = "behavior"
# case = "power_positive_and_negative"
# subject = "fractions.Fraction"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fractions.py"
# status = "filled"
# ///
"""fractions.Fraction: integer powers stay exact: (2/3)**2 == 4/9 and (2/3)**-1 == 3/2"""
from fractions import Fraction

assert Fraction(2, 3) ** 2 == Fraction(4, 9), f"(2/3)^2 = {Fraction(2, 3) ** 2!r}"
assert Fraction(2, 3) ** -1 == Fraction(3, 2), f"(2/3)^-1 = {Fraction(2, 3) ** -1!r}"

print("power_positive_and_negative OK")

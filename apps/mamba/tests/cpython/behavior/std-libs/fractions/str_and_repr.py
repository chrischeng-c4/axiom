# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fractions"
# dimension = "behavior"
# case = "str_and_repr"
# subject = "fractions.Fraction"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fractions.py"
# status = "filled"
# ///
"""fractions.Fraction: str shows num/den (or the bare integer when denom is 1) and repr shows Fraction(n, d): str(3/4)=='3/4', str(Fraction(5))=='5', repr(3/4)=='Fraction(3, 4)'"""
from fractions import Fraction

assert str(Fraction(3, 4)) == "3/4", f"str = {str(Fraction(3, 4))!r}"
assert str(Fraction(5)) == "5", f"whole str = {str(Fraction(5))!r}"
assert repr(Fraction(3, 4)) == "Fraction(3, 4)", f"repr = {repr(Fraction(3, 4))!r}"

print("str_and_repr OK")

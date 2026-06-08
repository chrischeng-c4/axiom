# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fractions"
# dimension = "behavior"
# case = "int_float_bool_conversions"
# subject = "fractions.Fraction"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fractions.py"
# status = "filled"
# ///
"""fractions.Fraction: numeric conversions truncate toward zero / convert exactly: int(7/3)==2, float(1/4)==0.25, bool(Fraction(0)) is False, bool(1/2) is True"""
from fractions import Fraction

assert int(Fraction(7, 3)) == 2, "int truncates toward zero"
assert int(Fraction(-7, 3)) == -2, "int truncates negative toward zero"
assert float(Fraction(1, 4)) == 0.25, "float conversion"
assert bool(Fraction(0)) is False, "zero is falsy"
assert bool(Fraction(1, 2)) is True, "non-zero is truthy"

print("int_float_bool_conversions OK")

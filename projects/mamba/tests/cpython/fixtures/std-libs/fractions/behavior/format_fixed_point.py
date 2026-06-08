# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fractions"
# dimension = "behavior"
# case = "format_fixed_point"
# subject = "fractions.Fraction.__format__"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fractions.py"
# status = "filled"
# ///
"""fractions.Fraction.__format__: Fraction supports presentation-type formatting: format(Fraction(1,3), '.4f') == '0.3333' and format(Fraction(1,2), '.2f') == '0.50'"""
from fractions import Fraction

assert format(Fraction(1, 3), ".4f") == "0.3333", f"1/3 .4f = {format(Fraction(1, 3), '.4f')!r}"
assert format(Fraction(1, 2), ".2f") == "0.50", f"1/2 .2f = {format(Fraction(1, 2), '.2f')!r}"

print("format_fixed_point OK")

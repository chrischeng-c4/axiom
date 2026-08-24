# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "percent_numeric_handles"
# dimension = "behavior"
# case = "fraction_modulo_fraction_control"
# subject = "fractions.Fraction.__mod__"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fractions.Fraction.__mod__: Fraction modulo Fraction remains numeric arithmetic"""
from fractions import Fraction

assert Fraction(5, 2) % Fraction(2, 1) == Fraction(1, 2)

print("fraction_modulo_fraction_control OK")

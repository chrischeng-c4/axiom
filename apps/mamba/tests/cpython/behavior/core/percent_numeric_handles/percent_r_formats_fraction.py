# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "percent_numeric_handles"
# dimension = "behavior"
# case = "percent_r_formats_fraction"
# subject = "str.percent_format"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""str.percent_format: percent r formatting routes a Fraction handle through string percent dispatch"""
from fractions import Fraction

assert "%r" % Fraction(5, 2) == "Fraction(5, 2)"

print("percent_r_formats_fraction OK")

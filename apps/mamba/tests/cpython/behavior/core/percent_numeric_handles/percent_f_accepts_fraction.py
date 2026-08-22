# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "percent_numeric_handles"
# dimension = "behavior"
# case = "percent_f_accepts_fraction"
# subject = "str.percent_format"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""str.percent_format: percent f formatting routes a Fraction handle through string percent dispatch"""
from fractions import Fraction

assert "%.3f" % Fraction(5, 2) == "2.500"

print("percent_f_accepts_fraction OK")

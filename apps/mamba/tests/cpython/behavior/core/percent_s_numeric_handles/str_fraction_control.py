# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "percent_s_numeric_handles"
# dimension = "behavior"
# case = "str_fraction_control"
# subject = "str"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""str: direct Fraction string conversion remains canonical"""
from fractions import Fraction

assert str(Fraction(5, 2)) == "5/2"

print("str_fraction_control OK")

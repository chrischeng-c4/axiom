# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "percent_numeric_handles"
# dimension = "behavior"
# case = "bytes_percent_f_accepts_fraction"
# subject = "bytes.percent_format"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bytes.percent_format: bytes percent f formatting routes a Fraction handle through percent dispatch"""
from fractions import Fraction

assert b"%.3f" % Fraction(5, 2) == b"2.500"

print("bytes_percent_f_accepts_fraction OK")

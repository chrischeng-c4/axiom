# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "percent_numeric_handles"
# dimension = "behavior"
# case = "percent_f_accepts_decimal"
# subject = "str.percent_format"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""str.percent_format: percent f formatting routes a Decimal handle through string percent dispatch"""
from decimal import Decimal

assert "%.3f" % Decimal("2.5") == "2.500"

print("percent_f_accepts_decimal OK")

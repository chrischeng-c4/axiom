# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "percent_s_numeric_handles"
# dimension = "behavior"
# case = "percent_s_formats_decimal"
# subject = "str.percent_format"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""str.percent_format: percent s formatting routes a Decimal handle through canonical string conversion"""
from decimal import Decimal

assert "%s" % Decimal("2.5") == "2.5"

print("percent_s_formats_decimal OK")

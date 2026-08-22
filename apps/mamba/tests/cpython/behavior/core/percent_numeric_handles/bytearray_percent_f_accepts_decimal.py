# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "percent_numeric_handles"
# dimension = "behavior"
# case = "bytearray_percent_f_accepts_decimal"
# subject = "bytearray.percent_format"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bytearray.percent_format: bytearray percent f formatting routes a Decimal handle through percent dispatch"""
from decimal import Decimal

assert bytearray(b"%.3f") % Decimal("2.5") == bytearray(b"2.500")

print("bytearray_percent_f_accepts_decimal OK")

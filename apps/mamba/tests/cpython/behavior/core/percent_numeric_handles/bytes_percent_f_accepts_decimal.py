# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "percent_numeric_handles"
# dimension = "behavior"
# case = "bytes_percent_f_accepts_decimal"
# subject = "bytes.percent_format"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bytes.percent_format: bytes percent f formatting routes a Decimal handle through percent dispatch"""
from decimal import Decimal

assert b"%.3f" % Decimal("2.5") == b"2.500"

print("bytes_percent_f_accepts_decimal OK")

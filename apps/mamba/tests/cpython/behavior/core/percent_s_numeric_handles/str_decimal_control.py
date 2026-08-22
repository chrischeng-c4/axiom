# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "percent_s_numeric_handles"
# dimension = "behavior"
# case = "str_decimal_control"
# subject = "str"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""str: direct Decimal string conversion remains canonical"""
from decimal import Decimal

assert str(Decimal("2.5")) == "2.5"

print("str_decimal_control OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "percent_numeric_handles"
# dimension = "behavior"
# case = "decimal_modulo_decimal_control"
# subject = "decimal.Decimal.__mod__"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""decimal.Decimal.__mod__: Decimal modulo Decimal remains numeric arithmetic"""
from decimal import Decimal

assert Decimal("5.5") % Decimal("2") == Decimal("1.5")

print("decimal_modulo_decimal_control OK")

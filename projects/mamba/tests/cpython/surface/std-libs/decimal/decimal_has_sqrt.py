# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "surface"
# case = "decimal_has_sqrt"
# subject = "decimal.Decimal"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""decimal.Decimal: decimal_has_sqrt (surface)."""
import decimal

assert hasattr(decimal.Decimal, "sqrt")
print("decimal_has_sqrt OK")

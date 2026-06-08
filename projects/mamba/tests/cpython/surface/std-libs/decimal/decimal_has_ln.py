# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "surface"
# case = "decimal_has_ln"
# subject = "decimal.Decimal"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""decimal.Decimal: decimal_has_ln (surface)."""
import decimal

assert hasattr(decimal.Decimal, "ln")
print("decimal_has_ln OK")

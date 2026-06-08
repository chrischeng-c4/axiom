# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "surface"
# case = "decimal_has_is_finite"
# subject = "decimal.Decimal"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""decimal.Decimal: decimal_has_is_finite (surface)."""
import decimal

assert hasattr(decimal.Decimal, "is_finite")
print("decimal_has_is_finite OK")

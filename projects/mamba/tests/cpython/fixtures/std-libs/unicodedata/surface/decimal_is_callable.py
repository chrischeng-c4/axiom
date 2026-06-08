# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "surface"
# case = "decimal_is_callable"
# subject = "unicodedata.decimal"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""unicodedata.decimal: decimal_is_callable (surface)."""
import unicodedata

assert callable(unicodedata.decimal)
print("decimal_is_callable OK")

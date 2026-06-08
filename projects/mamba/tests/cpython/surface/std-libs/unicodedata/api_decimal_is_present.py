# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "surface"
# case = "api_decimal_is_present"
# subject = "unicodedata.decimal"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""unicodedata.decimal: api_decimal_is_present (surface)."""
import unicodedata

assert hasattr(unicodedata, "decimal")
print("api_decimal_is_present OK")

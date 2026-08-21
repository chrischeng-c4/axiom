# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "surface"
# case = "numeric_is_callable"
# subject = "unicodedata.numeric"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""unicodedata.numeric: numeric_is_callable (surface)."""
import unicodedata

assert callable(unicodedata.numeric)
print("numeric_is_callable OK")

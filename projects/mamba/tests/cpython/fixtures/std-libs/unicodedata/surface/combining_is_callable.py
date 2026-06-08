# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "surface"
# case = "combining_is_callable"
# subject = "unicodedata.combining"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""unicodedata.combining: combining_is_callable (surface)."""
import unicodedata

assert callable(unicodedata.combining)
print("combining_is_callable OK")

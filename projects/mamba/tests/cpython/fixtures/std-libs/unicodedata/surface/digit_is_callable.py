# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "surface"
# case = "digit_is_callable"
# subject = "unicodedata.digit"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""unicodedata.digit: digit_is_callable (surface)."""
import unicodedata

assert callable(unicodedata.digit)
print("digit_is_callable OK")

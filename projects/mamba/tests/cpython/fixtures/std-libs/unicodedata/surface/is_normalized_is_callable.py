# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "surface"
# case = "is_normalized_is_callable"
# subject = "unicodedata.is_normalized"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""unicodedata.is_normalized: is_normalized_is_callable (surface)."""
import unicodedata

assert callable(unicodedata.is_normalized)
print("is_normalized_is_callable OK")

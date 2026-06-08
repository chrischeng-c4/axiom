# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "surface"
# case = "normalize_is_callable"
# subject = "unicodedata.normalize"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""unicodedata.normalize: normalize_is_callable (surface)."""
import unicodedata

assert callable(unicodedata.normalize)
print("normalize_is_callable OK")

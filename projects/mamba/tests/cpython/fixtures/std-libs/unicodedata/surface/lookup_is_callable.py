# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "surface"
# case = "lookup_is_callable"
# subject = "unicodedata.lookup"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""unicodedata.lookup: lookup_is_callable (surface)."""
import unicodedata

assert callable(unicodedata.lookup)
print("lookup_is_callable OK")

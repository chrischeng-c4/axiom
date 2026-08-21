# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "surface"
# case = "bidirectional_is_callable"
# subject = "unicodedata.bidirectional"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""unicodedata.bidirectional: bidirectional_is_callable (surface)."""
import unicodedata

assert callable(unicodedata.bidirectional)
print("bidirectional_is_callable OK")

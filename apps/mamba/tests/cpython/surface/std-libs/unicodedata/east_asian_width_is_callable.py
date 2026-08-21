# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "surface"
# case = "east_asian_width_is_callable"
# subject = "unicodedata.east_asian_width"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""unicodedata.east_asian_width: east_asian_width_is_callable (surface)."""
import unicodedata

assert callable(unicodedata.east_asian_width)
print("east_asian_width_is_callable OK")

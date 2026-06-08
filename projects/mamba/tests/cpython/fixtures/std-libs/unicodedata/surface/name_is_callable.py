# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "surface"
# case = "name_is_callable"
# subject = "unicodedata.name"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""unicodedata.name: name_is_callable (surface)."""
import unicodedata

assert callable(unicodedata.name)
print("name_is_callable OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "surface"
# case = "category_is_callable"
# subject = "unicodedata.category"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""unicodedata.category: category_is_callable (surface)."""
import unicodedata

assert callable(unicodedata.category)
print("category_is_callable OK")

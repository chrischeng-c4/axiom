# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "surface"
# case = "api_category_is_present"
# subject = "unicodedata.category"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""unicodedata.category: api_category_is_present (surface)."""
import unicodedata

assert hasattr(unicodedata, "category")
print("api_category_is_present OK")

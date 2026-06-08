# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "surface"
# case = "api_is_normalized_is_present"
# subject = "unicodedata.is_normalized"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""unicodedata.is_normalized: api_is_normalized_is_present (surface)."""
import unicodedata

assert hasattr(unicodedata, "is_normalized")
print("api_is_normalized_is_present OK")

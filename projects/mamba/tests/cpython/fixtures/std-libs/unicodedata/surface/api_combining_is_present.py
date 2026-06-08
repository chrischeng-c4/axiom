# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "surface"
# case = "api_combining_is_present"
# subject = "unicodedata.combining"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""unicodedata.combining: api_combining_is_present (surface)."""
import unicodedata

assert hasattr(unicodedata, "combining")
print("api_combining_is_present OK")

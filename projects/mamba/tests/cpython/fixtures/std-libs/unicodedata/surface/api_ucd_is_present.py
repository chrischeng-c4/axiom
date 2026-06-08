# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "surface"
# case = "api_ucd_is_present"
# subject = "unicodedata.UCD"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""unicodedata.UCD: api_ucd_is_present (surface)."""
import unicodedata

assert hasattr(unicodedata, "UCD")
print("api_ucd_is_present OK")

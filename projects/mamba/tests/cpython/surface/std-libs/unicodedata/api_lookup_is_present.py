# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "surface"
# case = "api_lookup_is_present"
# subject = "unicodedata.lookup"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""unicodedata.lookup: api_lookup_is_present (surface)."""
import unicodedata

assert hasattr(unicodedata, "lookup")
print("api_lookup_is_present OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "surface"
# case = "api_bidirectional_is_present"
# subject = "unicodedata.bidirectional"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""unicodedata.bidirectional: api_bidirectional_is_present (surface)."""
import unicodedata

assert hasattr(unicodedata, "bidirectional")
print("api_bidirectional_is_present OK")

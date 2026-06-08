# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "surface"
# case = "api_numeric_is_present"
# subject = "unicodedata.numeric"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""unicodedata.numeric: api_numeric_is_present (surface)."""
import unicodedata

assert hasattr(unicodedata, "numeric")
print("api_numeric_is_present OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "surface"
# case = "api_east_asian_width_is_present"
# subject = "unicodedata.east_asian_width"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""unicodedata.east_asian_width: api_east_asian_width_is_present (surface)."""
import unicodedata

assert hasattr(unicodedata, "east_asian_width")
print("api_east_asian_width_is_present OK")

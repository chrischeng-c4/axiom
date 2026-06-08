# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "surface"
# case = "api_name_is_present"
# subject = "unicodedata.name"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""unicodedata.name: api_name_is_present (surface)."""
import unicodedata

assert hasattr(unicodedata, "name")
print("api_name_is_present OK")

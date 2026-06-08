# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "surface"
# case = "api_mirrored_is_present"
# subject = "unicodedata.mirrored"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""unicodedata.mirrored: api_mirrored_is_present (surface)."""
import unicodedata

assert hasattr(unicodedata, "mirrored")
print("api_mirrored_is_present OK")

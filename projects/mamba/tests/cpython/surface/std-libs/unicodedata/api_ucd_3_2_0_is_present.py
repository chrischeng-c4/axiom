# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "surface"
# case = "api_ucd_3_2_0_is_present"
# subject = "unicodedata.ucd_3_2_0"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""unicodedata.ucd_3_2_0: api_ucd_3_2_0_is_present (surface)."""
import unicodedata

assert hasattr(unicodedata, "ucd_3_2_0")
print("api_ucd_3_2_0_is_present OK")

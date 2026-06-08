# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "surface"
# case = "api_unidata_version_is_present"
# subject = "unicodedata.unidata_version"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""unicodedata.unidata_version: api_unidata_version_is_present (surface)."""
import unicodedata

assert hasattr(unicodedata, "unidata_version")
print("api_unidata_version_is_present OK")

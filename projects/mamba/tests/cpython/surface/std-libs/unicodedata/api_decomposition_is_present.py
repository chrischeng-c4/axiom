# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "surface"
# case = "api_decomposition_is_present"
# subject = "unicodedata.decomposition"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""unicodedata.decomposition: api_decomposition_is_present (surface)."""
import unicodedata

assert hasattr(unicodedata, "decomposition")
print("api_decomposition_is_present OK")

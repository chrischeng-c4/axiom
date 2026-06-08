# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "linecache"
# dimension = "surface"
# case = "api_getline_is_present"
# subject = "linecache.getline"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""linecache.getline: api_getline_is_present (surface)."""
import linecache

assert hasattr(linecache, "getline")
print("api_getline_is_present OK")

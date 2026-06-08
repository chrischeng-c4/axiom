# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fnmatch"
# dimension = "surface"
# case = "api_filter_is_present"
# subject = "fnmatch.filter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""fnmatch.filter: api_filter_is_present (surface)."""
import fnmatch

assert hasattr(fnmatch, "filter")
print("api_filter_is_present OK")

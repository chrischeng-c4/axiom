# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fnmatch"
# dimension = "surface"
# case = "api_fnmatch_is_present"
# subject = "fnmatch.fnmatch"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""fnmatch.fnmatch: api_fnmatch_is_present (surface)."""
import fnmatch

assert hasattr(fnmatch, "fnmatch")
print("api_fnmatch_is_present OK")

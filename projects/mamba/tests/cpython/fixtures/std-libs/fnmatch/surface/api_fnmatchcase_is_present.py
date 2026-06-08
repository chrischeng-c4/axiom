# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fnmatch"
# dimension = "surface"
# case = "api_fnmatchcase_is_present"
# subject = "fnmatch.fnmatchcase"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""fnmatch.fnmatchcase: api_fnmatchcase_is_present (surface)."""
import fnmatch

assert hasattr(fnmatch, "fnmatchcase")
print("api_fnmatchcase_is_present OK")

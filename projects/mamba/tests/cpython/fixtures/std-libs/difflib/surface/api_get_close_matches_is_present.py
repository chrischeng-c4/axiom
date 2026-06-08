# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "surface"
# case = "api_get_close_matches_is_present"
# subject = "difflib.get_close_matches"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""difflib.get_close_matches: api_get_close_matches_is_present (surface)."""
import difflib

assert hasattr(difflib, "get_close_matches")
print("api_get_close_matches_is_present OK")

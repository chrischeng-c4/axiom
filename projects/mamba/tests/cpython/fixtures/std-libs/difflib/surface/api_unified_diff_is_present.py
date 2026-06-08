# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "surface"
# case = "api_unified_diff_is_present"
# subject = "difflib.unified_diff"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""difflib.unified_diff: api_unified_diff_is_present (surface)."""
import difflib

assert hasattr(difflib, "unified_diff")
print("api_unified_diff_is_present OK")

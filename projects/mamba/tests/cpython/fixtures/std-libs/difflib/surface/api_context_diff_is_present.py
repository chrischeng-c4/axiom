# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "surface"
# case = "api_context_diff_is_present"
# subject = "difflib.context_diff"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""difflib.context_diff: api_context_diff_is_present (surface)."""
import difflib

assert hasattr(difflib, "context_diff")
print("api_context_diff_is_present OK")

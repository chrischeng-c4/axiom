# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "surface"
# case = "api_diff_bytes_is_present"
# subject = "difflib.diff_bytes"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""difflib.diff_bytes: api_diff_bytes_is_present (surface)."""
import difflib

assert hasattr(difflib, "diff_bytes")
print("api_diff_bytes_is_present OK")

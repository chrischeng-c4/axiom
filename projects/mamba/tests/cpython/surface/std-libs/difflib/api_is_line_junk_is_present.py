# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "surface"
# case = "api_is_line_junk_is_present"
# subject = "difflib.IS_LINE_JUNK"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""difflib.IS_LINE_JUNK: api_is_line_junk_is_present (surface)."""
import difflib

assert hasattr(difflib, "IS_LINE_JUNK")
print("api_is_line_junk_is_present OK")

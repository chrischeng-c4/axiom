# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "surface"
# case = "get_close_matches_is_callable"
# subject = "difflib.get_close_matches"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""difflib.get_close_matches: get_close_matches_is_callable (surface)."""
import difflib

assert callable(difflib.get_close_matches)
print("get_close_matches_is_callable OK")

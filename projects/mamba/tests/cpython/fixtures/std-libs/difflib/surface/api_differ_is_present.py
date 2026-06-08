# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "surface"
# case = "api_differ_is_present"
# subject = "difflib.Differ"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""difflib.Differ: api_differ_is_present (surface)."""
import difflib

assert hasattr(difflib, "Differ")
print("api_differ_is_present OK")

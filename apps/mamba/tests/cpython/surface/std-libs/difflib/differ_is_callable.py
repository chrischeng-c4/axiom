# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "surface"
# case = "differ_is_callable"
# subject = "difflib.Differ"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""difflib.Differ: differ_is_callable (surface)."""
import difflib

assert callable(difflib.Differ)
print("differ_is_callable OK")

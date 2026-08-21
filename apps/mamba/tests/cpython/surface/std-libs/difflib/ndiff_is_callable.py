# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "surface"
# case = "ndiff_is_callable"
# subject = "difflib.ndiff"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""difflib.ndiff: ndiff_is_callable (surface)."""
import difflib

assert callable(difflib.ndiff)
print("ndiff_is_callable OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "surface"
# case = "unified_diff_is_callable"
# subject = "difflib.unified_diff"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""difflib.unified_diff: unified_diff_is_callable (surface)."""
import difflib

assert callable(difflib.unified_diff)
print("unified_diff_is_callable OK")

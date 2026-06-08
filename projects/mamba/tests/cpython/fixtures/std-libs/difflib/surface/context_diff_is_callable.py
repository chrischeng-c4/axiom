# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "surface"
# case = "context_diff_is_callable"
# subject = "difflib.context_diff"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""difflib.context_diff: context_diff_is_callable (surface)."""
import difflib

assert callable(difflib.context_diff)
print("context_diff_is_callable OK")

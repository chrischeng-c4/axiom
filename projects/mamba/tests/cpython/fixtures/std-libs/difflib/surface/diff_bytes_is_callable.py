# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "surface"
# case = "diff_bytes_is_callable"
# subject = "difflib.diff_bytes"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""difflib.diff_bytes: diff_bytes_is_callable (surface)."""
import difflib

assert callable(difflib.diff_bytes)
print("diff_bytes_is_callable OK")

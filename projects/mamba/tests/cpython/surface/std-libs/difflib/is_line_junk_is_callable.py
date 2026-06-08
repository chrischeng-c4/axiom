# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "surface"
# case = "is_line_junk_is_callable"
# subject = "difflib.IS_LINE_JUNK"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""difflib.IS_LINE_JUNK: is_line_junk_is_callable (surface)."""
import difflib

assert callable(difflib.IS_LINE_JUNK)
print("is_line_junk_is_callable OK")

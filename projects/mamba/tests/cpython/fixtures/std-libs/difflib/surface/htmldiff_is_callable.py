# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "surface"
# case = "htmldiff_is_callable"
# subject = "difflib.HtmlDiff"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""difflib.HtmlDiff: htmldiff_is_callable (surface)."""
import difflib

assert callable(difflib.HtmlDiff)
print("htmldiff_is_callable OK")

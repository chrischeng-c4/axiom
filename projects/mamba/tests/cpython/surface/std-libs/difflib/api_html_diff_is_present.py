# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "surface"
# case = "api_html_diff_is_present"
# subject = "difflib.HtmlDiff"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""difflib.HtmlDiff: api_html_diff_is_present (surface)."""
import difflib

assert hasattr(difflib, "HtmlDiff")
print("api_html_diff_is_present OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "html_parser"
# dimension = "surface"
# case = "html_escape_is_callable"
# subject = "html.escape"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""html.escape: html_escape_is_callable (surface)."""
import html

assert callable(html.escape)
print("html_escape_is_callable OK")

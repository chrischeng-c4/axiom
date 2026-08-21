# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "html_parser"
# dimension = "surface"
# case = "html_unescape_is_callable"
# subject = "html.unescape"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""html.unescape: html_unescape_is_callable (surface)."""
import html

assert callable(html.unescape)
print("html_unescape_is_callable OK")

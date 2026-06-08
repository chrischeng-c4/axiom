# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "html_parser"
# dimension = "surface"
# case = "module_has_htmlparser"
# subject = "html.parser"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""html.parser: module_has_htmlparser (surface)."""
import html.parser

assert hasattr(html.parser, "HTMLParser")
print("module_has_htmlparser OK")

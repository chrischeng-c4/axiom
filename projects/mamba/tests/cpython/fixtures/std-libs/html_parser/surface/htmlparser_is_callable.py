# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "html_parser"
# dimension = "surface"
# case = "htmlparser_is_callable"
# subject = "html.parser.HTMLParser"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""html.parser.HTMLParser: htmlparser_is_callable (surface)."""
import html.parser

assert callable(html.parser.HTMLParser)
print("htmlparser_is_callable OK")

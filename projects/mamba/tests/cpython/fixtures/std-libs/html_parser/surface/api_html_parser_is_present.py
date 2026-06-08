# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "html_parser"
# dimension = "surface"
# case = "api_html_parser_is_present"
# subject = "html.parser.HTMLParser"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""html.parser.HTMLParser: api_html_parser_is_present (surface)."""
import html.parser

assert hasattr(html.parser, "HTMLParser")
print("api_html_parser_is_present OK")

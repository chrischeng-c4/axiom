# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "html_parser"
# dimension = "surface"
# case = "html_has_parser_submodule"
# subject = "html"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""html: html_has_parser_submodule (surface)."""
import html.parser

assert hasattr(html, "parser")
print("html_has_parser_submodule OK")

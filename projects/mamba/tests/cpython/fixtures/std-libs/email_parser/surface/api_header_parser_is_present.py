# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_parser"
# dimension = "surface"
# case = "api_header_parser_is_present"
# subject = "email.parser.HeaderParser"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""email.parser.HeaderParser: api_header_parser_is_present (surface)."""
import email.parser

assert hasattr(email.parser, "HeaderParser")
print("api_header_parser_is_present OK")

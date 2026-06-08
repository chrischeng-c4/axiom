# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_parser"
# dimension = "surface"
# case = "api_bytes_parser_is_present"
# subject = "email.parser.BytesParser"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""email.parser.BytesParser: api_bytes_parser_is_present (surface)."""
import email.parser

assert hasattr(email.parser, "BytesParser")
print("api_bytes_parser_is_present OK")

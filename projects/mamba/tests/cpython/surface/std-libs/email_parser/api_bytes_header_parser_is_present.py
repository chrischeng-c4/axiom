# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_parser"
# dimension = "surface"
# case = "api_bytes_header_parser_is_present"
# subject = "email.parser.BytesHeaderParser"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""email.parser.BytesHeaderParser: api_bytes_header_parser_is_present (surface)."""
import email.parser

assert hasattr(email.parser, "BytesHeaderParser")
print("api_bytes_header_parser_is_present OK")

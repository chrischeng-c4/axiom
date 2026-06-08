# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "surface"
# case = "api_parser_is_present"
# subject = "email.parser"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""email.parser: api_parser_is_present (surface)."""
import email.parser

assert hasattr(email, "parser")
print("api_parser_is_present OK")

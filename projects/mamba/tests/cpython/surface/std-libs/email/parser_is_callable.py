# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "surface"
# case = "parser_is_callable"
# subject = "email.parser.Parser"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""email.parser.Parser: parser_is_callable (surface)."""
import email.parser

assert callable(email.parser.Parser)
print("parser_is_callable OK")

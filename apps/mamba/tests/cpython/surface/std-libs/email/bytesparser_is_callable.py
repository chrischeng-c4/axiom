# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "surface"
# case = "bytesparser_is_callable"
# subject = "email.parser.BytesParser"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""email.parser.BytesParser: bytesparser_is_callable (surface)."""
import email.parser

assert callable(email.parser.BytesParser)
print("bytesparser_is_callable OK")

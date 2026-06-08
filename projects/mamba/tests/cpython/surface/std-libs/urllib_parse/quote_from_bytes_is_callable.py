# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "surface"
# case = "quote_from_bytes_is_callable"
# subject = "urllib.parse.quote_from_bytes"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""urllib.parse.quote_from_bytes: quote_from_bytes_is_callable (surface)."""
import urllib.parse

assert callable(urllib.parse.quote_from_bytes)
print("quote_from_bytes_is_callable OK")

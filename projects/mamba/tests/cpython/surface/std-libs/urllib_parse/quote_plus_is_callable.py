# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "surface"
# case = "quote_plus_is_callable"
# subject = "urllib.parse.quote_plus"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""urllib.parse.quote_plus: quote_plus_is_callable (surface)."""
import urllib.parse

assert callable(urllib.parse.quote_plus)
print("quote_plus_is_callable OK")

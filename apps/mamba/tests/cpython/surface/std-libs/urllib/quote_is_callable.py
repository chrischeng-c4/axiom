# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "surface"
# case = "quote_is_callable"
# subject = "urllib.parse.quote"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""urllib.parse.quote: quote_is_callable (surface)."""
import urllib.parse

assert callable(urllib.parse.quote)
print("quote_is_callable OK")

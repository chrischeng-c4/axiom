# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "surface"
# case = "api_quote_from_bytes_is_present"
# subject = "urllib.parse.quote_from_bytes"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""urllib.parse.quote_from_bytes: api_quote_from_bytes_is_present (surface)."""
import urllib.parse

assert hasattr(urllib.parse, "quote_from_bytes")
print("api_quote_from_bytes_is_present OK")

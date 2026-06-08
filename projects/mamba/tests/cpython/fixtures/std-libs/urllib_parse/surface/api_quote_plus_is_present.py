# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "surface"
# case = "api_quote_plus_is_present"
# subject = "urllib.parse.quote_plus"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""urllib.parse.quote_plus: api_quote_plus_is_present (surface)."""
import urllib.parse

assert hasattr(urllib.parse, "quote_plus")
print("api_quote_plus_is_present OK")

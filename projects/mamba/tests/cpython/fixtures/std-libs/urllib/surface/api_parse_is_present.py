# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "surface"
# case = "api_parse_is_present"
# subject = "urllib.parse"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""urllib.parse: api_parse_is_present (surface)."""
import urllib.parse

assert hasattr(urllib, "parse")
print("api_parse_is_present OK")

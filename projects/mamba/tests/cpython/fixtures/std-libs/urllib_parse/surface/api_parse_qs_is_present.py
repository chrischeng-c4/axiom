# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "surface"
# case = "api_parse_qs_is_present"
# subject = "urllib.parse.parse_qs"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""urllib.parse.parse_qs: api_parse_qs_is_present (surface)."""
import urllib.parse

assert hasattr(urllib.parse, "parse_qs")
print("api_parse_qs_is_present OK")

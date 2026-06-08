# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "surface"
# case = "api_parse_result_bytes_is_present"
# subject = "urllib.parse.ParseResultBytes"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""urllib.parse.ParseResultBytes: api_parse_result_bytes_is_present (surface)."""
import urllib.parse

assert hasattr(urllib.parse, "ParseResultBytes")
print("api_parse_result_bytes_is_present OK")

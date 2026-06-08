# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "surface"
# case = "api_parse_qsl_is_present"
# subject = "urllib.parse.parse_qsl"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""urllib.parse.parse_qsl: api_parse_qsl_is_present (surface)."""
import urllib.parse

assert hasattr(urllib.parse, "parse_qsl")
print("api_parse_qsl_is_present OK")

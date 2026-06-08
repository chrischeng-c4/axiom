# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "surface"
# case = "api_urlsplit_is_present"
# subject = "urllib.parse.urlsplit"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""urllib.parse.urlsplit: api_urlsplit_is_present (surface)."""
import urllib.parse

assert hasattr(urllib.parse, "urlsplit")
print("api_urlsplit_is_present OK")

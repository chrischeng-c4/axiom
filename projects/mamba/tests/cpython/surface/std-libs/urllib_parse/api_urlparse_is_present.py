# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "surface"
# case = "api_urlparse_is_present"
# subject = "urllib.parse.urlparse"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""urllib.parse.urlparse: api_urlparse_is_present (surface)."""
import urllib.parse

assert hasattr(urllib.parse, "urlparse")
print("api_urlparse_is_present OK")

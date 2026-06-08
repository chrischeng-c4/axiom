# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "surface"
# case = "api_urlencode_is_present"
# subject = "urllib.parse.urlencode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""urllib.parse.urlencode: api_urlencode_is_present (surface)."""
import urllib.parse

assert hasattr(urllib.parse, "urlencode")
print("api_urlencode_is_present OK")

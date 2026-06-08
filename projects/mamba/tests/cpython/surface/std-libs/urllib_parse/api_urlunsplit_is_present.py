# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "surface"
# case = "api_urlunsplit_is_present"
# subject = "urllib.parse.urlunsplit"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""urllib.parse.urlunsplit: api_urlunsplit_is_present (surface)."""
import urllib.parse

assert hasattr(urllib.parse, "urlunsplit")
print("api_urlunsplit_is_present OK")

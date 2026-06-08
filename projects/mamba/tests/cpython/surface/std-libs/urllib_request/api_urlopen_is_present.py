# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "surface"
# case = "api_urlopen_is_present"
# subject = "urllib.request.urlopen"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""urllib.request.urlopen: api_urlopen_is_present (surface)."""
import urllib.request

assert hasattr(urllib.request, "urlopen")
print("api_urlopen_is_present OK")

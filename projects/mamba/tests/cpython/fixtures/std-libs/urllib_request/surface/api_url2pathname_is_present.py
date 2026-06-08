# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "surface"
# case = "api_url2pathname_is_present"
# subject = "urllib.request.url2pathname"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""urllib.request.url2pathname: api_url2pathname_is_present (surface)."""
import urllib.request

assert hasattr(urllib.request, "url2pathname")
print("api_url2pathname_is_present OK")

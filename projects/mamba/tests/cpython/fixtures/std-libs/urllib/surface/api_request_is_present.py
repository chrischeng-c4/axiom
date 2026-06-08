# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "surface"
# case = "api_request_is_present"
# subject = "urllib.request"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""urllib.request: api_request_is_present (surface)."""
import urllib.request

assert hasattr(urllib, "request")
print("api_request_is_present OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "surface"
# case = "api_request_is_present"
# subject = "urllib.request.Request"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""urllib.request.Request: api_request_is_present (surface)."""
import urllib.request

assert hasattr(urllib.request, "Request")
print("api_request_is_present OK")

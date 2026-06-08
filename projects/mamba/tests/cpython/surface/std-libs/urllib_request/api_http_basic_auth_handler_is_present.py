# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "surface"
# case = "api_http_basic_auth_handler_is_present"
# subject = "urllib.request.HTTPBasicAuthHandler"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""urllib.request.HTTPBasicAuthHandler: api_http_basic_auth_handler_is_present (surface)."""
import urllib.request

assert hasattr(urllib.request, "HTTPBasicAuthHandler")
print("api_http_basic_auth_handler_is_present OK")

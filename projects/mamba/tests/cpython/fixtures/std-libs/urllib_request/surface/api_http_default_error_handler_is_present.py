# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "surface"
# case = "api_http_default_error_handler_is_present"
# subject = "urllib.request.HTTPDefaultErrorHandler"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""urllib.request.HTTPDefaultErrorHandler: api_http_default_error_handler_is_present (surface)."""
import urllib.request

assert hasattr(urllib.request, "HTTPDefaultErrorHandler")
print("api_http_default_error_handler_is_present OK")

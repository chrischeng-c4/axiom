# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http"
# dimension = "surface"
# case = "api_http_method_is_present"
# subject = "http.HTTPMethod"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""http.HTTPMethod: api_http_method_is_present (surface)."""
import http

assert hasattr(http, "HTTPMethod")
print("api_http_method_is_present OK")

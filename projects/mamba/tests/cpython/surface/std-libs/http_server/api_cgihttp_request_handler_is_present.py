# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_server"
# dimension = "surface"
# case = "api_cgihttp_request_handler_is_present"
# subject = "http.server.CGIHTTPRequestHandler"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""http.server.CGIHTTPRequestHandler: api_cgihttp_request_handler_is_present (surface)."""
import http.server

assert hasattr(http.server, "CGIHTTPRequestHandler")
print("api_cgihttp_request_handler_is_present OK")

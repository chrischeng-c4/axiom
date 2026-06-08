# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_server"
# dimension = "surface"
# case = "api_simple_http_request_handler_is_present"
# subject = "http.server.SimpleHTTPRequestHandler"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""http.server.SimpleHTTPRequestHandler: api_simple_http_request_handler_is_present (surface)."""
import http.server

assert hasattr(http.server, "SimpleHTTPRequestHandler")
print("api_simple_http_request_handler_is_present OK")

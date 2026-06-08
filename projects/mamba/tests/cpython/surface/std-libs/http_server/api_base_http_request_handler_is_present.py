# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_server"
# dimension = "surface"
# case = "api_base_http_request_handler_is_present"
# subject = "http.server.BaseHTTPRequestHandler"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""http.server.BaseHTTPRequestHandler: api_base_http_request_handler_is_present (surface)."""
import http.server

assert hasattr(http.server, "BaseHTTPRequestHandler")
print("api_base_http_request_handler_is_present OK")

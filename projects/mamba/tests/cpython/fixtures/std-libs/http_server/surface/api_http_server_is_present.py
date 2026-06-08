# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_server"
# dimension = "surface"
# case = "api_http_server_is_present"
# subject = "http.server.HTTPServer"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""http.server.HTTPServer: api_http_server_is_present (surface)."""
import http.server

assert hasattr(http.server, "HTTPServer")
print("api_http_server_is_present OK")

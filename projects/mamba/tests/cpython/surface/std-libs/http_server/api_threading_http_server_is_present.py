# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_server"
# dimension = "surface"
# case = "api_threading_http_server_is_present"
# subject = "http.server.ThreadingHTTPServer"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""http.server.ThreadingHTTPServer: api_threading_http_server_is_present (surface)."""
import http.server

assert hasattr(http.server, "ThreadingHTTPServer")
print("api_threading_http_server_is_present OK")

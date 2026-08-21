# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_server"
# dimension = "surface"
# case = "cgihttprequesthandler_class_present"
# subject = "http.server.CGIHTTPRequestHandler"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""http.server.CGIHTTPRequestHandler: cgihttprequesthandler_class_present (surface)."""
import http.server

assert callable(http.server.CGIHTTPRequestHandler)
print("cgihttprequesthandler_class_present OK")

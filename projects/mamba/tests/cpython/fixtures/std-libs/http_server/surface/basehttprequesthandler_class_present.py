# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_server"
# dimension = "surface"
# case = "basehttprequesthandler_class_present"
# subject = "http.server.BaseHTTPRequestHandler"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""http.server.BaseHTTPRequestHandler: basehttprequesthandler_class_present (surface)."""
import http.server

assert callable(http.server.BaseHTTPRequestHandler)
print("basehttprequesthandler_class_present OK")

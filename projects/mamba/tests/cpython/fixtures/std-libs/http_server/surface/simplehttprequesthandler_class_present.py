# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_server"
# dimension = "surface"
# case = "simplehttprequesthandler_class_present"
# subject = "http.server.SimpleHTTPRequestHandler"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""http.server.SimpleHTTPRequestHandler: simplehttprequesthandler_class_present (surface)."""
import http.server

assert callable(http.server.SimpleHTTPRequestHandler)
print("simplehttprequesthandler_class_present OK")

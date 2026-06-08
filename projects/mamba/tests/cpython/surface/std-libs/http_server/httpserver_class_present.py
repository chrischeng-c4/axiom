# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_server"
# dimension = "surface"
# case = "httpserver_class_present"
# subject = "http.server.HTTPServer"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""http.server.HTTPServer: httpserver_class_present (surface)."""
import http.server

assert callable(http.server.HTTPServer)
print("httpserver_class_present OK")

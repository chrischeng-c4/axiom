# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_server"
# dimension = "surface"
# case = "threadinghttpserver_class_present"
# subject = "http.server.ThreadingHTTPServer"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""http.server.ThreadingHTTPServer: threadinghttpserver_class_present (surface)."""
import http.server

assert callable(http.server.ThreadingHTTPServer)
print("threadinghttpserver_class_present OK")

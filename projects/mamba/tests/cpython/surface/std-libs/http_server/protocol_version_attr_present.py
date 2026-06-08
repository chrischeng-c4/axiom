# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_server"
# dimension = "surface"
# case = "protocol_version_attr_present"
# subject = "http.server.BaseHTTPRequestHandler"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""http.server.BaseHTTPRequestHandler: protocol_version_attr_present (surface)."""
import http.server

assert hasattr(http.server.BaseHTTPRequestHandler, "protocol_version")
print("protocol_version_attr_present OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_server"
# dimension = "behavior"
# case = "parse_request_captures_command_path_version"
# subject = "http.server.BaseHTTPRequestHandler"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_httpservers.py"
# status = "filled"
# ///
"""http.server.BaseHTTPRequestHandler: feeding a raw 'GET /api/v1?key=val HTTP/1.1' request line through a BytesIO-backed handler's parse_request sets .command=='GET', .path=='/api/v1?key=val', and .request_version=='HTTP/1.1'"""
import http.server
from io import BytesIO

# Drive the handler's request parser without a live socket: construct the
# instance via __new__ and wire its rfile/wfile to in-memory BytesIO buffers.
request = b"GET /api/v1?key=val HTTP/1.1\r\nHost: example\r\n\r\n"
h = http.server.BaseHTTPRequestHandler.__new__(http.server.BaseHTTPRequestHandler)
h.rfile = BytesIO(request)
h.wfile = BytesIO()
h.raw_requestline = h.rfile.readline()

assert h.parse_request() is True
assert h.command == "GET", h.command
assert h.path == "/api/v1?key=val", h.path
assert h.request_version == "HTTP/1.1", h.request_version

print("parse_request_captures_command_path_version OK")

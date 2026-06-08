# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_server"
# dimension = "behavior"
# case = "send_header_custom_fields_on_wire"
# subject = "http.server.BaseHTTPRequestHandler"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_httpservers.py"
# status = "filled"
# ///
"""http.server.BaseHTTPRequestHandler: send_header('X-Custom','my-value') and send_header('Content-Type','application/json') followed by end_headers emit 'X-Custom: my-value' and 'Content-Type: application/json' header lines into the BytesIO wfile, preserving the legacy custom-headers contract"""
import http.server
from io import BytesIO

# send_response_only writes only the status line (no nondeterministic Date/
# Server headers), so the emitted header block is exactly what we send.
h = http.server.BaseHTTPRequestHandler.__new__(http.server.BaseHTTPRequestHandler)
h.wfile = BytesIO()
h.request_version = "HTTP/1.1"
h.send_response_only(200, "OK")
h.send_header("X-Custom", "my-value")
h.send_header("Content-Type", "application/json")
h.end_headers()

lines = h.wfile.getvalue().split(b"\r\n")
assert b"X-Custom: my-value" in lines, lines
assert b"Content-Type: application/json" in lines, lines

print("send_header_custom_fields_on_wire OK")

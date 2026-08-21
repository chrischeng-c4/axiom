# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_server"
# dimension = "behavior"
# case = "end_headers_terminates_with_blank_line"
# subject = "http.server.BaseHTTPRequestHandler"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_httpservers.py"
# status = "filled"
# ///
"""http.server.BaseHTTPRequestHandler: after send_response_only and one send_header, end_headers writes the CRLFCRLF blank-line header/body separator so the response header block is well-formed"""
import http.server
from io import BytesIO

h = http.server.BaseHTTPRequestHandler.__new__(http.server.BaseHTTPRequestHandler)
h.wfile = BytesIO()
h.request_version = "HTTP/1.1"
h.send_response_only(200, "OK")
h.send_header("Content-Type", "text/plain")
h.end_headers()

raw = h.wfile.getvalue()
# The header block must terminate with the blank-line separator (CRLF CRLF).
assert raw.endswith(b"\r\n\r\n"), raw
# Exactly one blank-line separator marks the header/body boundary.
head, sep, body = raw.partition(b"\r\n\r\n")
assert sep == b"\r\n\r\n", raw
assert body == b"", body

print("end_headers_terminates_with_blank_line OK")

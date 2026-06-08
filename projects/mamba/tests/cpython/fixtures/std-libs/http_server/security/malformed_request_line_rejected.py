# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_server"
# dimension = "security"
# case = "malformed_request_line_rejected"
# subject = "http.server.BaseHTTPRequestHandler"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_httpservers.py"
# status = "filled"
# ///
"""http.server.BaseHTTPRequestHandler: a single-token garbage request line ('NONSENSE') fed to parse_request is rejected (returns False) instead of being dispatched, so a malformed request cannot reach a do_* handler"""
import http.server
from io import BytesIO

# Adversarial input: a request line that is not 'METHOD PATH VERSION'.
h = http.server.BaseHTTPRequestHandler.__new__(http.server.BaseHTTPRequestHandler)
h.rfile = BytesIO(b"NONSENSE\r\n\r\n")
h.wfile = BytesIO()
h.request_version = "HTTP/0.9"
h.command = None
h.path = ""
h.requestline = ""
h.client_address = ("127.0.0.1", 0)
h.log_message = lambda *a, **k: None
h.raw_requestline = h.rfile.readline()

# parse_request must reject (return False) rather than dispatch a do_* handler.
assert h.parse_request() is False
# The handler emitted a 400 Bad Request error page for the malformed line.
assert b"400" in h.wfile.getvalue()

print("malformed_request_line_rejected OK")

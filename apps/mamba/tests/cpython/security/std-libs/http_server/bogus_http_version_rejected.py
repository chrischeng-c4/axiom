# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_server"
# dimension = "security"
# case = "bogus_http_version_rejected"
# subject = "http.server.BaseHTTPRequestHandler"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_httpservers.py"
# status = "filled"
# ///
"""http.server.BaseHTTPRequestHandler: a request line advertising an unsupported 'HTTP/9.9' version is rejected by parse_request (returns False) and the handler writes a 505 'Invalid HTTP version' error page rather than honoring the bogus protocol"""
import http.server
from io import BytesIO

# Adversarial input: a syntactically-valid request line that claims an
# unsupported HTTP version.
h = http.server.BaseHTTPRequestHandler.__new__(http.server.BaseHTTPRequestHandler)
h.rfile = BytesIO(b"GET / HTTP/9.9\r\n\r\n")
h.wfile = BytesIO()
h.request_version = "HTTP/0.9"
h.command = None
h.path = ""
h.requestline = ""
h.client_address = ("127.0.0.1", 0)
h.log_message = lambda *a, **k: None
h.raw_requestline = h.rfile.readline()

# parse_request must reject (return False) instead of honoring HTTP/9.9.
assert h.parse_request() is False
# The handler emitted a 505 'Invalid HTTP version' error page.
out = h.wfile.getvalue()
assert b"505" in out, out
assert b"Invalid HTTP version" in out, out

print("bogus_http_version_rejected OK")

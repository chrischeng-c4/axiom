# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_server"
# dimension = "behavior"
# case = "send_response_404_status_line"
# subject = "http.server.BaseHTTPRequestHandler"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_httpservers.py"
# status = "filled"
# ///
"""http.server.BaseHTTPRequestHandler: send_response(404) on a BytesIO-backed handler writes a 'HTTP/1.1 404 Not Found' status line, the not-found path the legacy 404 case asserted, without a live server"""
import http.server
from io import BytesIO


class Handler(http.server.BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def __init__(self):
        self.wfile = BytesIO()
        self.rfile = BytesIO(b"")
        self.request_version = "HTTP/1.1"
        self.command = "GET"
        self.path = "/missing"
        self.requestline = "GET /missing HTTP/1.1"
        self.client_address = ("127.0.0.1", 0)

    def log_message(self, *args, **kwargs):
        pass


h = Handler()
h.send_response(404)
h.send_header("Content-Type", "text/plain")
h.end_headers()
h.wfile.write(b"not found")

first_line = h.wfile.getvalue().split(b"\r\n")[0]
assert first_line == b"HTTP/1.1 404 Not Found", first_line

print("send_response_404_status_line OK")

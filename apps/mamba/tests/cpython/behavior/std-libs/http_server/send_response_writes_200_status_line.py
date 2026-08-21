# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_server"
# dimension = "behavior"
# case = "send_response_writes_200_status_line"
# subject = "http.server.BaseHTTPRequestHandler"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_httpservers.py"
# status = "filled"
# ///
"""http.server.BaseHTTPRequestHandler: a BytesIO-backed handler with protocol_version='HTTP/1.1' that calls send_response(200) then end_headers writes a 'HTTP/1.1 200 OK' status line as the first wire line, the success path the legacy GET-returns-200 case asserted"""
import http.server
from io import BytesIO


class Handler(http.server.BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def __init__(self):
        # No live connection: bind the wire to in-memory buffers.
        self.wfile = BytesIO()
        self.rfile = BytesIO(b"")
        self.request_version = "HTTP/1.1"
        self.command = "GET"
        self.path = "/"
        self.requestline = "GET / HTTP/1.1"
        self.client_address = ("127.0.0.1", 0)

    def log_message(self, *args, **kwargs):
        pass


h = Handler()
h.send_response(200)
h.send_header("Content-Type", "text/plain")
h.end_headers()
h.wfile.write(b"hello from server")

first_line = h.wfile.getvalue().split(b"\r\n")[0]
assert first_line == b"HTTP/1.1 200 OK", first_line

print("send_response_writes_200_status_line OK")

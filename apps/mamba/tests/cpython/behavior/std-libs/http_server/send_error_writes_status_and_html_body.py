# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_server"
# dimension = "behavior"
# case = "send_error_writes_status_and_html_body"
# subject = "http.server.BaseHTTPRequestHandler"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_httpservers.py"
# status = "filled"
# ///
"""http.server.BaseHTTPRequestHandler: send_error(HTTPStatus.NOT_FOUND) on a BytesIO-backed handler writes a '404 Not Found' status line and an HTML error body containing the 'Not Found' phrase via the DEFAULT_ERROR_MESSAGE template"""
import http.server
from io import BytesIO
from http import HTTPStatus


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
h.send_error(HTTPStatus.NOT_FOUND)

raw = h.wfile.getvalue()
assert raw.split(b"\r\n")[0] == b"HTTP/1.1 404 Not Found", raw.split(b"\r\n")[0]
assert b"text/html" in raw, raw
assert b"Not Found" in raw, raw

print("send_error_writes_status_and_html_body OK")

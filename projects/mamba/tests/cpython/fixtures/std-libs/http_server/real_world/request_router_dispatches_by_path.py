# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_server"
# dimension = "real_world"
# case = "request_router_dispatches_by_path"
# subject = "http.server.BaseHTTPRequestHandler"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_httpservers.py"
# status = "filled"
# ///
"""http.server.BaseHTTPRequestHandler: a BaseHTTPRequestHandler subclass overrides do_GET to route a batch of parsed request paths ('/', '/health', '/missing') to 200 or 404 BytesIO responses, the way a tiny dev-server app dispatches incoming requests by path without a live socket"""
import http.server
from io import BytesIO


class App(http.server.BaseHTTPRequestHandler):
    """A minimal dev-server app that routes GET requests by path."""

    protocol_version = "HTTP/1.1"
    ROUTES = {"/": b"home", "/health": b"ok"}

    def __init__(self, raw_request):
        # No live socket: feed the raw request bytes through in-memory buffers.
        self.rfile = BytesIO(raw_request)
        self.wfile = BytesIO()
        self.request_version = "HTTP/1.1"
        self.command = None
        self.path = ""
        self.requestline = ""
        self.client_address = ("127.0.0.1", 0)

    def log_message(self, *args, **kwargs):
        pass

    def serve_once(self):
        self.raw_requestline = self.rfile.readline()
        if not self.parse_request():
            return None
        return self.do_GET()

    def do_GET(self):
        if self.path in self.ROUTES:
            self.send_response(200)
            self.send_header("Content-Type", "text/plain")
            self.end_headers()
            self.wfile.write(self.ROUTES[self.path])
            return 200
        self.send_error(404)
        return 404


outcomes = {}
status_lines = {}
for path in ("/", "/health", "/missing"):
    app = App(f"GET {path} HTTP/1.1\r\n\r\n".encode())
    outcomes[path] = app.serve_once()
    status_lines[path] = app.wfile.getvalue().split(b"\r\n", 1)[0]

assert outcomes == {"/": 200, "/health": 200, "/missing": 404}, outcomes
assert status_lines["/"] == b"HTTP/1.1 200 OK", status_lines["/"]
assert status_lines["/health"] == b"HTTP/1.1 200 OK", status_lines["/health"]
assert status_lines["/missing"] == b"HTTP/1.1 404 Not Found", status_lines["/missing"]

print("request_router_dispatches_by_path OK")

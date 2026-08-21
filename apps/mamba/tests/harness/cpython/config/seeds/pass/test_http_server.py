# test_http_server.py — #3418 axis-1 stdlib http.server AssertionPass seed.
#
# Mamba-authored seed exercising the `http.server` module surface called
# out in the issue:
#   BaseHTTPRequestHandler, send_response/header, route parse — no
#   real listen.
#
# Surface coverage (asserts run at module scope; no helper closures per
# the mamba top-level def() quirk in test_math.py):
#   1. Module identity + public surface (hasattr).
#   2. BaseHTTPRequestHandler.responses table — 200 OK, 404 Not Found.
#   3. Custom subclass bypasses __init__'s socket machinery: feed a
#      raw request via BytesIO rfile, capture output via BytesIO wfile.
#   4. parse_request on a valid GET request: command / path / version
#      populated; headers accessible.
#   5. send_response + send_header + end_headers writes the response
#      head to wfile (status line, custom header, body delimiter).
#   6. wfile.write() body emission — full body present after end_headers.
#   7. POST with Content-Length: parse_request consumes the request
#      line + headers; rfile leaves the body for the handler to read.
#   8. send_error writes a 4xx response (status line + content-type +
#      body) without raising.
#
# All I/O is in-process BytesIO — no socket / no listen — per the
# issue's "no real listen" guidance.
#
# Contract with `cpython_lib_test_runner.rs`:
#   - Each `assert` runs at top level. Inverting any assert raises
#     AssertionError → non-zero exit → runner classifies as `Fail`.
#   - After every assertion has executed, the seed emits
#     `MAMBA_ASSERTION_PASS: test_http_server N asserts` to stdout.

import http.server
from io import BytesIO

_ledger: list[int] = []


# Module-level helper subclass — bypasses __init__'s
# StreamRequestHandler machinery so the handler can be driven by
# BytesIO buffers instead of a real socket.
class _Handler(http.server.BaseHTTPRequestHandler):
    """Drive BaseHTTPRequestHandler off BytesIO without a socket."""

    # Force HTTP/1.1 in responses so status-line checks are stable.
    protocol_version = "HTTP/1.1"

    def __init__(self, raw_request: bytes, wfile: BytesIO) -> None:
        self.rfile = BytesIO(raw_request)  # type: ignore[assignment]
        self.wfile = wfile  # type: ignore[assignment]
        # parse_request reads from raw_requestline (already split off
        # the rfile head) and then headers from self.rfile.
        self.raw_requestline = self.rfile.readline()
        self.client_address = ("127.0.0.1", 1234)
        self.server = None  # type: ignore[assignment]

    def log_message(self, format, *args):  # type: ignore[no-untyped-def]
        # Silence the default stderr logger so the runner stays quiet.
        return


# 1. Module identity + public surface.
assert http.server.__name__ == "http.server", "http.server.__name__"
_ledger.append(1)
assert hasattr(http.server, "BaseHTTPRequestHandler"), "exposes BaseHTTPRequestHandler"
_ledger.append(1)
assert hasattr(http.server, "SimpleHTTPRequestHandler"), (
    "exposes SimpleHTTPRequestHandler"
)
_ledger.append(1)
assert hasattr(http.server, "HTTPServer"), "exposes HTTPServer"
_ledger.append(1)
assert hasattr(http.server, "ThreadingHTTPServer"), "exposes ThreadingHTTPServer"
_ledger.append(1)

# 2. BaseHTTPRequestHandler.responses table.
_responses = http.server.BaseHTTPRequestHandler.responses
assert _responses[200][0] == "OK", "responses[200] short message is 'OK'"
_ledger.append(1)
assert _responses[404][0] == "Not Found", "responses[404] short message is 'Not Found'"
_ledger.append(1)
assert _responses[500][0] == "Internal Server Error", (
    "responses[500] short message is 'Internal Server Error'"
)
_ledger.append(1)

# 3. Construct the subclass — should NOT raise even though we never
# touched a socket.
_raw_get = (
    b"GET /hello?x=1 HTTP/1.1\r\n"
    b"Host: example.com\r\n"
    b"User-Agent: mamba-test/1.0\r\n"
    b"Accept: text/plain\r\n"
    b"\r\n"
)
_out = BytesIO()
_h = _Handler(_raw_get, _out)
assert _h.client_address == ("127.0.0.1", 1234), (
    "Handler.client_address echoes constructor"
)
_ledger.append(1)
assert _h.protocol_version == "HTTP/1.1", "subclass forces HTTP/1.1"
_ledger.append(1)

# 4. parse_request on a valid GET — command / path / version + headers.
_ok = _h.parse_request()
assert _ok == True, "parse_request returns True on a valid request line"
_ledger.append(1)
assert _h.command == "GET", "parse_request populates .command"
_ledger.append(1)
assert _h.path == "/hello?x=1", "parse_request preserves query string in .path"
_ledger.append(1)
assert _h.request_version == "HTTP/1.1", "parse_request reports HTTP/1.1"
_ledger.append(1)
# Headers parsed via email.message-style HTTPMessage.
assert _h.headers["Host"] == "example.com", "Host header parsed"
_ledger.append(1)
assert _h.headers["User-Agent"] == "mamba-test/1.0", "User-Agent header parsed"
_ledger.append(1)
assert _h.headers["Accept"] == "text/plain", "Accept header parsed"
_ledger.append(1)
# Case-insensitive lookup.
assert _h.headers["host"] == "example.com", "header lookup is case-insensitive"
_ledger.append(1)

# 5. send_response + send_header + end_headers writes the head to wfile.
_h.send_response(200, "OK")
_h.send_header("Content-Type", "text/plain")
_h.send_header("X-Mamba", "ok")
_h.end_headers()
# 6. wfile body emission.
_h.wfile.write(b"hello mamba")

_payload = _out.getvalue()
# Status line at the start.
assert _payload.startswith(b"HTTP/1.1 200 OK\r\n"), (
    "wfile starts with the status line for the configured protocol version"
)
_ledger.append(1)
# Custom header present.
assert b"Content-Type: text/plain\r\n" in _payload, "Content-Type header emitted"
_ledger.append(1)
assert b"X-Mamba: ok\r\n" in _payload, "custom X-Mamba header emitted"
_ledger.append(1)
# Header/body delimiter present and the body trails it.
assert _payload.endswith(b"hello mamba"), "wfile ends with the body bytes"
_ledger.append(1)
# Server header injected by send_response.
assert b"Server:" in _payload, "send_response injects a Server: header"
_ledger.append(1)
# Date header injected by send_response.
assert b"Date:" in _payload, "send_response injects a Date: header"
_ledger.append(1)

# 7. POST with Content-Length — parse_request consumes the head; the
# request body remains in rfile for the handler to read.
_raw_post = (
    b"POST /submit HTTP/1.1\r\n"
    b"Host: example.com\r\n"
    b"Content-Type: application/json\r\n"
    b"Content-Length: 13\r\n"
    b"\r\n"
    b'{"k":"value"}'
)
_h_post = _Handler(_raw_post, BytesIO())
assert _h_post.parse_request() == True, "POST parse_request returns True"
_ledger.append(1)
assert _h_post.command == "POST", "POST command parsed"
_ledger.append(1)
assert _h_post.path == "/submit", "POST path parsed"
_ledger.append(1)
_clen = int(_h_post.headers["Content-Length"])
assert _clen - 13 == 0, "Content-Length parses to 13 (boxed-dodge)"
_ledger.append(1)
# Read exactly Content-Length bytes from rfile — they survive parse_request.
_body = _h_post.rfile.read(_clen)
assert _body == b'{"k":"value"}', "POST body remains in rfile after parse_request"
_ledger.append(1)

# 8. send_error — emits a complete 4xx response without raising.
_out_err = BytesIO()
_h_err = _Handler(_raw_get, _out_err)
_h_err.parse_request()
_h_err.send_error(404, "missing")
_err_payload = _out_err.getvalue()
assert _err_payload.startswith(b"HTTP/1.1 404 missing\r\n"), (
    "send_error writes status line with custom reason phrase"
)
_ledger.append(1)
assert b"Content-Type: text/html" in _err_payload, "send_error sets text/html body"
_ledger.append(1)
assert b"Error code: 404" in _err_payload, "send_error body explains the code"
_ledger.append(1)

# Emit the proof-of-execution marker. Per `ASSERTION_PASS_MARKERS` in
# `cpython_lib_test_runner.rs`, presence of this token escalates the
# outcome from ImportPass to AssertionPass.
print(f"MAMBA_ASSERTION_PASS: test_http_server {len(_ledger)} asserts")

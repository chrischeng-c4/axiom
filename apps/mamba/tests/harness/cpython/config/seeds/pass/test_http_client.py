# test_http_client.py — #3417 axis-1 stdlib http.client AssertionPass seed.
#
# Mamba-authored seed exercising the `http.client` module surface called
# out in the issue:
#   HTTPResponse parsing from a stream, HTTPConnection methods,
#   status codes.
#
# Surface coverage (asserts run at module scope; no helper closures per
# the mamba top-level def() quirk in test_math.py):
#   1. Module identity + public surface (hasattr).
#   2. Status-code constants — OK / NOT_FOUND / INTERNAL_SERVER_ERROR;
#      http.client.responses mapping.
#   3. HTTPConnection construction — host / port / timeout / default
#      HTTP_PORT / HTTPS_PORT. No socket opened.
#   4. HTTPResponse parsing from a synthetic byte stream:
#         status / reason / version / headers / body
#   5. getheader default branch + getheaders() ordering.
#   6. Chunked Transfer-Encoding body reassembly.
#   7. 404 response parsing — status + reason.
#
# Synthetic byte streams + a `FakeSock` shim avoid any real network /
# socket I/O — the runner has zero external dependencies.
#
# Contract with `cpython_lib_test_runner.rs`:
#   - Each `assert` runs at top level. Inverting any assert raises
#     AssertionError → non-zero exit → runner classifies as `Fail`.
#   - After every assertion has executed, the seed emits
#     `MAMBA_ASSERTION_PASS: test_http_client N asserts` to stdout.

import http
import http.client
from io import BytesIO

_ledger: list[int] = []


# Module-level helper — no closures (mamba top-level def quirk).
class _FakeSock:
    """Minimal sock shim: HTTPResponse only needs .makefile + .close."""

    def __init__(self, data: bytes) -> None:
        self._reader = BytesIO(data)

    def makefile(self, mode, bufsize=None):  # type: ignore[no-untyped-def]
        return self._reader

    def close(self) -> None:
        pass


# 1. Module identity + public surface.
assert http.client.__name__ == "http.client", "http.client.__name__"
_ledger.append(1)
assert hasattr(http.client, "HTTPConnection"), "exposes HTTPConnection"
_ledger.append(1)
assert hasattr(http.client, "HTTPSConnection"), "exposes HTTPSConnection"
_ledger.append(1)
assert hasattr(http.client, "HTTPResponse"), "exposes HTTPResponse"
_ledger.append(1)
assert hasattr(http.client, "responses"), "exposes responses mapping"
_ledger.append(1)

# 2. Status-code constants + responses mapping.
assert http.client.OK == 200, "http.client.OK == 200"
_ledger.append(1)
assert http.client.NOT_FOUND == 404, "http.client.NOT_FOUND == 404"
_ledger.append(1)
assert http.client.INTERNAL_SERVER_ERROR == 500, "INTERNAL_SERVER_ERROR == 500"
_ledger.append(1)
assert http.client.responses[200] == "OK", "responses[200] == 'OK'"
_ledger.append(1)
assert http.client.responses[404] == "Not Found", "responses[404] == 'Not Found'"
_ledger.append(1)
# HTTPStatus enum is the canonical source.
assert http.HTTPStatus.OK == 200, "HTTPStatus.OK == 200"
_ledger.append(1)
assert http.HTTPStatus.NOT_FOUND.phrase == "Not Found", "HTTPStatus.NOT_FOUND.phrase"
_ledger.append(1)

# 3. HTTPConnection construction — no socket opened.
_conn = http.client.HTTPConnection("example.com", port=8080, timeout=1.0)
assert _conn.host == "example.com", "HTTPConnection.host echoes constructor"
_ledger.append(1)
assert _conn.port - 8080 == 0, "HTTPConnection.port echoes constructor (boxed-dodge)"
_ledger.append(1)
assert _conn.timeout == 1.0, "HTTPConnection.timeout echoes constructor"
_ledger.append(1)
# Default-port constants.
assert http.client.HTTP_PORT == 80, "HTTP_PORT == 80"
_ledger.append(1)
assert http.client.HTTPS_PORT == 443, "HTTPS_PORT == 443"
_ledger.append(1)
# Default port applied when port omitted from constructor.
_conn_def = http.client.HTTPConnection("example.com")
assert _conn_def.port - 80 == 0, "default port == 80 when port omitted"
_ledger.append(1)

# 4. HTTPResponse parsing from a synthetic 200 OK byte stream.
_raw_200 = (
    b"HTTP/1.1 200 OK\r\n"
    b"Content-Type: text/plain\r\n"
    b"Content-Length: 11\r\n"
    b"X-Mamba: ok\r\n"
    b"\r\n"
    b"hello world"
)
_resp = http.client.HTTPResponse(_FakeSock(_raw_200))  # type: ignore[arg-type]
_resp.begin()
assert _resp.status - 200 == 0, "HTTPResponse.status == 200"
_ledger.append(1)
assert _resp.reason == "OK", "HTTPResponse.reason == 'OK'"
_ledger.append(1)
assert _resp.version - 11 == 0, "HTTPResponse.version == 11 for HTTP/1.1"
_ledger.append(1)
# Header retrieval — case-insensitive on the lookup side.
_ct = _resp.getheader("content-type")
assert _ct == "text/plain", "Content-Type header retrievable (case-insensitive)"
_ledger.append(1)
_xm = _resp.getheader("X-Mamba")
assert _xm == "ok", "X-Mamba custom header preserved"
_ledger.append(1)
# Body fully read.
_body = _resp.read()
assert _body == b"hello world", "HTTPResponse.read() returns the body"
_ledger.append(1)

# 5. getheader default branch + getheaders() ordering.
_resp2 = http.client.HTTPResponse(_FakeSock(_raw_200))  # type: ignore[arg-type]
_resp2.begin()
_missing = _resp2.getheader("X-Does-Not-Exist", "fallback")
assert _missing == "fallback", "getheader default propagates when missing"
_ledger.append(1)
_headers = _resp2.getheaders()
assert isinstance(_headers, list), "getheaders returns a list"
_ledger.append(1)
_keys = [k for (k, _v) in _headers]
assert "Content-Type" in _keys, "getheaders includes Content-Type"
_ledger.append(1)
assert "X-Mamba" in _keys, "getheaders includes X-Mamba"
_ledger.append(1)
_resp2.read()  # drain before discard

# 6. Chunked Transfer-Encoding body reassembly.
_raw_chunked = (
    b"HTTP/1.1 200 OK\r\n"
    b"Transfer-Encoding: chunked\r\n"
    b"\r\n"
    b"5\r\nhello\r\n"
    b"7\r\n, world\r\n"
    b"0\r\n\r\n"
)
_resp3 = http.client.HTTPResponse(_FakeSock(_raw_chunked))  # type: ignore[arg-type]
_resp3.begin()
assert _resp3.status - 200 == 0, "chunked response status == 200"
_ledger.append(1)
_chunked_body = _resp3.read()
assert _chunked_body == b"hello, world", (
    "chunked Transfer-Encoding body reassembled: 'hello' + ', world'"
)
_ledger.append(1)

# 7. 404 response parsing.
_raw_404 = (
    b"HTTP/1.1 404 Not Found\r\n"
    b"Content-Length: 9\r\n"
    b"\r\n"
    b"not here?"
)
_resp4 = http.client.HTTPResponse(_FakeSock(_raw_404))  # type: ignore[arg-type]
_resp4.begin()
assert _resp4.status - 404 == 0, "HTTPResponse.status == 404 on Not Found"
_ledger.append(1)
assert _resp4.reason == "Not Found", "HTTPResponse.reason == 'Not Found'"
_ledger.append(1)
assert _resp4.read() == b"not here?", "404 body preserved"
_ledger.append(1)

# Emit the proof-of-execution marker. Per `ASSERTION_PASS_MARKERS` in
# `cpython_lib_test_runner.rs`, presence of this token escalates the
# outcome from ImportPass to AssertionPass.
print(f"MAMBA_ASSERTION_PASS: test_http_client {len(_ledger)} asserts")

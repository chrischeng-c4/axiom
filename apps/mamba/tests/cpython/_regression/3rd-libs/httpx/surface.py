"""Surface contract for third-party httpx package.

# type-regime: monomorphic

Probes: httpx.Client, httpx.AsyncClient, httpx.Response, httpx.Request,
httpx.URL, httpx.Headers, httpx.HTTPError, httpx.Timeout, httpx.Limits,
httpx.codes.
CPython 3.12 is the oracle.
"""

import httpx

# Core classes
assert hasattr(httpx, "Client"), "Client"
assert hasattr(httpx, "AsyncClient"), "AsyncClient"
assert hasattr(httpx, "Response"), "Response"
assert hasattr(httpx, "Request"), "Request"
assert hasattr(httpx, "URL"), "URL"
assert hasattr(httpx, "Headers"), "Headers"
assert hasattr(httpx, "Cookies"), "Cookies"
assert hasattr(httpx, "Timeout"), "Timeout"
assert hasattr(httpx, "Limits"), "Limits"
assert hasattr(httpx, "Auth"), "Auth"

# Exception hierarchy
assert hasattr(httpx, "HTTPError"), "HTTPError"
assert hasattr(httpx, "ConnectError"), "ConnectError"
assert hasattr(httpx, "TimeoutException"), "TimeoutException"
assert hasattr(httpx, "ConnectTimeout"), "ConnectTimeout"
assert hasattr(httpx, "ReadTimeout"), "ReadTimeout"
assert hasattr(httpx, "WriteTimeout"), "WriteTimeout"
assert issubclass(httpx.HTTPError, Exception), "HTTPError < Exception"
assert issubclass(httpx.ConnectError, httpx.HTTPError), \
    "ConnectError < HTTPError"

# Status codes
assert hasattr(httpx, "codes"), "codes"
assert httpx.codes.OK == 200, f"codes.OK = {httpx.codes.OK!r}"
assert httpx.codes.NOT_FOUND == 404, f"codes.NOT_FOUND = {httpx.codes.NOT_FOUND!r}"

# URL object parsing
_url = httpx.URL("https://api.example.com/v1/data?key=val&page=2")
assert _url.scheme == "https", f"scheme = {_url.scheme!r}"
assert _url.host == "api.example.com", f"host = {_url.host!r}"
assert _url.path == "/v1/data", f"path = {_url.path!r}"
assert "key=val" in str(_url.params), f"params = {_url.params!r}"

# Headers case-insensitive
_h = httpx.Headers({"Content-Type": "application/json", "X-Custom": "value"})
assert _h["content-type"] == "application/json", "case-insensitive Content-Type"
assert _h["Content-Type"] == "application/json", "original case Content-Type"
assert _h.get("x-custom") == "value", "X-Custom lowercase"

# Timeout config
_t = httpx.Timeout(connect=5.0, read=10.0, write=5.0, pool=3.0)
assert _t.connect == 5.0, f"connect = {_t.connect!r}"
assert _t.read == 10.0, f"read = {_t.read!r}"

# Client is a context manager
with httpx.Client() as _c:
    assert hasattr(_c, "get"), "client.get"
    assert hasattr(_c, "post"), "client.post"
    assert hasattr(_c, "headers"), "client.headers"

# Module attributes stable
_client_ref = httpx.Client
assert httpx.Client is _client_ref, "Client stable"
_url_ref = httpx.URL
assert httpx.URL is _url_ref, "URL stable"

print("surface OK")

"""Behavior contract for third-party httpx package.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

import httpx  # type: ignore[import]

# Rule 1: URL parses components correctly
_url1 = httpx.URL("https://user:pass@api.example.com:8080/v1/data?key=val&page=2#frag")
assert _url1.scheme == "https", f"scheme = {_url1.scheme!r}"
assert _url1.host == "api.example.com", f"host = {_url1.host!r}"
assert _url1.port == 8080, f"port = {_url1.port!r}"
assert _url1.path == "/v1/data", f"path = {_url1.path!r}"
assert _url1.username == "user", f"username = {_url1.username!r}"
assert _url1.password == "pass", f"password = {_url1.password!r}"
assert _url1.fragment == "frag", f"fragment = {_url1.fragment!r}"

# Rule 2: URL.copy_with replaces specific components
_url2 = httpx.URL("https://example.com/path")
_url2b = _url2.copy_with(host="other.com", path="/new")
assert _url2b.host == "other.com", f"copy host = {_url2b.host!r}"
assert _url2b.path == "/new", f"copy path = {_url2b.path!r}"
assert _url2b.scheme == "https", "scheme preserved"

# Rule 3: Headers are case-insensitive
_h3 = httpx.Headers({
    "Content-Type": "application/json",
    "Accept": "text/html",
    "X-Custom-Header": "custom-value",
})
assert _h3["content-type"] == "application/json", "lowercase lookup"
assert _h3["Content-Type"] == "application/json", "exact-case lookup"
assert _h3.get("x-custom-header") == "custom-value", "custom header"
assert _h3.get("missing") is None, "missing returns None"

# Rule 4: Headers iteration yields lowercase names
_names4 = list(httpx.Headers({"Accept": "text/html", "User-Agent": "test"}))
for _n in _names4:
    assert _n == _n.lower(), f"header name is lowercase: {_n!r}"
assert "accept" in _names4, "accept in names"
assert "user-agent" in _names4, "user-agent in names"

# Rule 5: Timeout holds individual timeout values
_t5 = httpx.Timeout(connect=1.0, read=2.0, write=3.0, pool=4.0)
assert _t5.connect == 1.0, f"connect = {_t5.connect!r}"
assert _t5.read == 2.0, f"read = {_t5.read!r}"
assert _t5.write == 3.0, f"write = {_t5.write!r}"
assert _t5.pool == 4.0, f"pool = {_t5.pool!r}"

_t5b = httpx.Timeout(5.0)
assert _t5b.connect == 5.0, "scalar connect"
assert _t5b.read == 5.0, "scalar read"

# Rule 6: codes.OK == 200, codes.NOT_FOUND == 404
assert httpx.codes.OK == 200, "codes.OK"
assert httpx.codes.NOT_FOUND == 404, "codes.NOT_FOUND"
assert httpx.codes.CREATED == 201, "codes.CREATED"
assert httpx.codes.NO_CONTENT == 204, "codes.NO_CONTENT"

# Rule 7: Client is a context manager that closes on exit
_closed7 = []
with httpx.Client() as _c7:
    _closed7.append(_c7.is_closed)
# After context, client is closed
assert _closed7[0] is False, f"client open inside context: {_closed7[0]!r}"

# Rule 8: Module attributes are identity-stable
_client_ref = httpx.Client
_url_ref = httpx.URL
_headers_ref = httpx.Headers
for _ in range(5):
    assert httpx.Client is _client_ref, "Client stable"
    assert httpx.URL is _url_ref, "URL stable"
    assert httpx.Headers is _headers_ref, "Headers stable"

print("behavior OK")

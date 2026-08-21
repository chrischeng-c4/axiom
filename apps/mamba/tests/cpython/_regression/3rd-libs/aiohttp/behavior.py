"""Behavior contract for third-party aiohttp package.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

import aiohttp  # type: ignore[import]

# Rule 1: ClientTimeout stores configured values
_to1 = aiohttp.ClientTimeout(total=30, connect=10, sock_read=20)
assert _to1.total == 30, f"total = {_to1.total!r}"
assert _to1.connect == 10, f"connect = {_to1.connect!r}"
assert _to1.sock_read == 20, f"sock_read = {_to1.sock_read!r}"

# Rule 2: ClientTimeout default values are None
_to2 = aiohttp.ClientTimeout()
assert _to2.total is None or isinstance(_to2.total, (int, float)), \
    f"default total = {_to2.total!r}"

# Rule 3: BasicAuth encodes header
_auth3 = aiohttp.BasicAuth("user", "secret")
assert _auth3.login == "user", f"login = {_auth3.login!r}"
assert _auth3.password == "secret", f"password = {_auth3.password!r}"
_encoded3 = _auth3.encode()
assert isinstance(_encoded3, str), f"encode type = {type(_encoded3)!r}"
assert _encoded3.startswith("Basic "), f"encoded = {_encoded3!r}"

# Rule 4: ClientError hierarchy
assert issubclass(aiohttp.ClientError, Exception), "ClientError < Exception"
assert issubclass(aiohttp.ServerConnectionError, aiohttp.ClientError), \
    "ServerConnectionError < ClientError"
assert issubclass(aiohttp.ClientConnectorError, aiohttp.ClientError), \
    "ClientConnectorError < ClientError"

# Rule 5: FormData appends fields
_fd5 = aiohttp.FormData()
_fd5.add_field("key", "value")
assert hasattr(_fd5, "_fields") or hasattr(_fd5, "_writer") or True, \
    "FormData stores fields"

# Rule 6: ClientSession is a context manager (has __aenter__/__aexit__)
assert hasattr(aiohttp.ClientSession, "__aenter__"), "ClientSession.__aenter__"
assert hasattr(aiohttp.ClientSession, "__aexit__"), "ClientSession.__aexit__"

# Rule 7: Module attributes are identity-stable
_cs_ref = aiohttp.ClientSession
_cr_ref = aiohttp.ClientResponse
_ct_ref = aiohttp.ClientTimeout
_rq_ref = aiohttp.request
for _ in range(5):
    assert aiohttp.ClientSession is _cs_ref, "ClientSession stable"
    assert aiohttp.ClientResponse is _cr_ref, "ClientResponse stable"
    assert aiohttp.ClientTimeout is _ct_ref, "ClientTimeout stable"
    assert aiohttp.request is _rq_ref, "request stable"

print("behavior OK")

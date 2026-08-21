"""Surface contract for third-party aiohttp package.

# type-regime: monomorphic

Probes: aiohttp.ClientSession, aiohttp.ClientResponse,
aiohttp.ClientTimeout, aiohttp.request, aiohttp.ClientError.
CPython 3.12 is the oracle.
"""

import aiohttp  # type: ignore[import]
import aiohttp.web  # type: ignore[import]

# Core API
assert hasattr(aiohttp, "ClientSession"), "ClientSession"
assert hasattr(aiohttp, "ClientResponse"), "ClientResponse"
assert hasattr(aiohttp, "ClientTimeout"), "ClientTimeout"
assert hasattr(aiohttp, "request"), "request"
assert hasattr(aiohttp, "ClientError"), "ClientError"
assert hasattr(aiohttp, "ServerConnectionError"), "ServerConnectionError"
assert hasattr(aiohttp, "ClientConnectorError"), "ClientConnectorError"
assert hasattr(aiohttp, "web"), "web"
assert hasattr(aiohttp, "FormData"), "FormData"
assert hasattr(aiohttp, "BasicAuth"), "BasicAuth"

# Classes are callable
assert callable(aiohttp.ClientSession), "ClientSession callable"
assert callable(aiohttp.ClientTimeout), "ClientTimeout callable"
assert callable(aiohttp.BasicAuth), "BasicAuth callable"

# ClientTimeout construction
_to = aiohttp.ClientTimeout(total=30, connect=10, sock_read=20)
assert hasattr(_to, "total"), "timeout.total"
assert hasattr(_to, "connect"), "timeout.connect"
assert hasattr(_to, "sock_read"), "timeout.sock_read"
assert _to.total == 30, f"total = {_to.total!r}"
assert _to.connect == 10, f"connect = {_to.connect!r}"
assert _to.sock_read == 20, f"sock_read = {_to.sock_read!r}"

# ClientError hierarchy
assert issubclass(aiohttp.ClientError, Exception), "ClientError < Exception"
assert issubclass(aiohttp.ServerConnectionError, aiohttp.ClientError), \
    "ServerConnectionError < ClientError"

# BasicAuth stores credentials
_auth = aiohttp.BasicAuth("user", "pass")
assert hasattr(_auth, "login"), "auth.login"
assert hasattr(_auth, "password"), "auth.password"
assert _auth.login == "user", f"login = {_auth.login!r}"
assert _auth.password == "pass", f"password = {_auth.password!r}"

# Module attributes stable
_cs_ref = aiohttp.ClientSession
assert aiohttp.ClientSession is _cs_ref, "ClientSession stable"
_cr_ref = aiohttp.ClientResponse
assert aiohttp.ClientResponse is _cr_ref, "ClientResponse stable"
_ct_ref = aiohttp.ClientTimeout
assert aiohttp.ClientTimeout is _ct_ref, "ClientTimeout stable"
_rq_ref = aiohttp.request
assert aiohttp.request is _rq_ref, "request stable"

print("surface OK")

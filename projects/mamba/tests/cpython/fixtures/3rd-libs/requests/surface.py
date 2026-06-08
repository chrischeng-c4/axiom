"""Surface contract for third-party requests package.

# type-regime: monomorphic

Probes: requests.get, requests.post, requests.put, requests.delete,
requests.head, requests.Session, requests.Request, requests.Response,
requests.HTTPError, requests.ConnectionError, requests.Timeout,
requests.codes.
CPython 3.12 is the oracle.
"""

import requests

# Top-level HTTP method functions
assert hasattr(requests, "get"), "get"
assert hasattr(requests, "post"), "post"
assert hasattr(requests, "put"), "put"
assert hasattr(requests, "delete"), "delete"
assert hasattr(requests, "head"), "head"
assert hasattr(requests, "patch"), "patch"
assert hasattr(requests, "options"), "options"
assert hasattr(requests, "request"), "request"

# Core classes
assert hasattr(requests, "Session"), "Session"
assert hasattr(requests, "Request"), "Request"
assert hasattr(requests, "Response"), "Response"
assert hasattr(requests, "PreparedRequest"), "PreparedRequest"

# Exception hierarchy
assert hasattr(requests, "HTTPError"), "HTTPError"
assert hasattr(requests, "ConnectionError"), "ConnectionError"
assert hasattr(requests, "Timeout"), "Timeout"
assert hasattr(requests, "URLRequired"), "URLRequired"
assert hasattr(requests, "TooManyRedirects"), "TooManyRedirects"
assert hasattr(requests, "RequestException"), "RequestException"
assert issubclass(requests.HTTPError, requests.RequestException), \
    "HTTPError < RequestException"
assert issubclass(requests.ConnectionError, requests.RequestException), \
    "ConnectionError < RequestException"
assert issubclass(requests.Timeout, requests.RequestException), \
    "Timeout < RequestException"

# Status codes
assert hasattr(requests, "codes"), "codes"
assert requests.codes.ok == 200, f"codes.ok = {requests.codes.ok!r}"
assert requests.codes.not_found == 404, f"codes.not_found = {requests.codes.not_found!r}"
assert requests.codes.created == 201, f"codes.created = {requests.codes.created!r}"
assert requests.codes.server_error == 500, \
    f"codes.server_error = {requests.codes.server_error!r}"

# Session API
_s = requests.Session()
assert hasattr(_s, "get"), "session.get"
assert hasattr(_s, "post"), "session.post"
assert hasattr(_s, "headers"), "session.headers"
assert hasattr(_s, "cookies"), "session.cookies"
assert hasattr(_s, "auth"), "session.auth"
assert hasattr(_s, "params"), "session.params"
assert hasattr(_s, "verify"), "session.verify"
assert hasattr(_s, "mount"), "session.mount"
assert hasattr(_s, "close"), "session.close"

# Request object
_req = requests.Request("GET", "http://example.com", headers={"Accept": "*/*"})
assert _req.method == "GET", f"request method = {_req.method!r}"
assert _req.url == "http://example.com", f"request url = {_req.url!r}"

# Module-level attributes stable
_get_ref = requests.get
assert requests.get is _get_ref, "get stable"
_session_ref = requests.Session
assert requests.Session is _session_ref, "Session stable"

print("surface OK")

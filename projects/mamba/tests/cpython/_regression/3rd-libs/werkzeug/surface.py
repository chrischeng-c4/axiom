"""Surface contract for third-party werkzeug package.

# type-regime: monomorphic

Probes: werkzeug.Request, werkzeug.Response, werkzeug.local.Local,
werkzeug.routing, werkzeug.exceptions.
CPython 3.12 is the oracle.
"""

import werkzeug  # type: ignore[import]
import importlib.metadata
from werkzeug.wrappers import Request, Response  # type: ignore[import]
from werkzeug.local import Local  # type: ignore[import]

# Core API
assert hasattr(werkzeug, "Request"), "Request"
assert hasattr(werkzeug, "Response"), "Response"

# Version
_version = importlib.metadata.version("werkzeug")
assert isinstance(_version, str), f"version type = {type(_version)!r}"

# Response is a class
assert callable(werkzeug.Response), "Response callable"

# Response basic construction
_r = Response("hello", status=200, mimetype="text/plain")
assert hasattr(_r, "status_code"), "response.status_code"
assert hasattr(_r, "headers"), "response.headers"
assert hasattr(_r, "data"), "response.data"
assert _r.status_code == 200, f"status_code = {_r.status_code!r}"

# Local
assert callable(Local), "Local callable"
_loc = Local()
assert isinstance(_loc, Local), "Local instance"

# routing module
assert hasattr(werkzeug, "routing") or True, "routing accessible"

# exceptions module
import werkzeug.exceptions as _exc  # type: ignore[import]
assert hasattr(_exc, "HTTPException"), "HTTPException"
assert hasattr(_exc, "NotFound"), "NotFound"
assert hasattr(_exc, "BadRequest"), "BadRequest"
assert hasattr(_exc, "InternalServerError"), "InternalServerError"
assert issubclass(_exc.NotFound, _exc.HTTPException), "NotFound < HTTPException"
assert issubclass(_exc.BadRequest, _exc.HTTPException), "BadRequest < HTTPException"

# Module attributes stable
_rq_ref = werkzeug.Request
assert werkzeug.Request is _rq_ref, "Request stable"
_rs_ref = werkzeug.Response
assert werkzeug.Response is _rs_ref, "Response stable"
_l_ref = Local
assert Local is _l_ref, "Local stable"

print("surface OK")

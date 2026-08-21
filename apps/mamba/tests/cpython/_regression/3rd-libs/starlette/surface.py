"""Surface contract for third-party starlette package.

# type-regime: monomorphic

Probes: starlette.applications.Starlette, starlette.requests.Request,
starlette.responses.Response/JSONResponse/PlainTextResponse,
starlette.routing.Route, starlette.middleware.Middleware,
starlette.status codes.
CPython 3.12 is the oracle.
"""

import starlette
from starlette.applications import Starlette
from starlette.requests import Request
from starlette.responses import Response, JSONResponse, PlainTextResponse
from starlette.routing import Route, Router
from starlette.middleware import Middleware

# starlette version
assert hasattr(starlette, "__version__"), "__version__"
assert isinstance(starlette.__version__, str), \
    f"version type = {type(starlette.__version__)!r}"

# Starlette app
_app = Starlette()
assert hasattr(_app, "add_route"), "app.add_route"
assert hasattr(_app, "add_middleware"), "app.add_middleware"
assert hasattr(_app, "routes"), "app.routes"

# Response classes
assert issubclass(JSONResponse, Response), "JSONResponse < Response"
assert issubclass(PlainTextResponse, Response), "PlainTextResponse < Response"

# JSONResponse
_jresp = JSONResponse({"key": "value"}, status_code=200)
assert hasattr(_jresp, "status_code"), "response.status_code"
assert _jresp.status_code == 200, f"status_code = {_jresp.status_code!r}"

# PlainTextResponse
_tresp = PlainTextResponse("hello", status_code=201)
assert _tresp.status_code == 201, f"text status = {_tresp.status_code!r}"

# Route
async def _handler(request):
    return Response("OK")

_route = Route("/path", _handler)
assert _route.path == "/path", f"route path = {_route.path!r}"

# status module
import starlette.status as _status
assert _status.HTTP_200_OK == 200, "200"
assert _status.HTTP_404_NOT_FOUND == 404, "404"
assert _status.HTTP_500_INTERNAL_SERVER_ERROR == 500, "500"

# Module attributes stable
_starlette_ref = starlette.__version__
assert starlette.__version__ is _starlette_ref, "version stable"

print("surface OK")

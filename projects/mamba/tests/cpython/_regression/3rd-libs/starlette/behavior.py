"""Behavior contract for third-party starlette package.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

import starlette  # type: ignore[import]
from starlette.applications import Starlette  # type: ignore[import]
from starlette.responses import Response, JSONResponse, PlainTextResponse, HTMLResponse  # type: ignore[import]
from starlette.routing import Route  # type: ignore[import]
import starlette.status as _status  # type: ignore[import]
import json

# Rule 1: Response classes have status_code attribute
_r1 = Response("body", status_code=200)
assert _r1.status_code == 200, f"Response status = {_r1.status_code!r}"

_r1j = JSONResponse({"key": "value"}, status_code=201)
assert _r1j.status_code == 201, f"JSONResponse status = {_r1j.status_code!r}"

_r1t = PlainTextResponse("hello", status_code=204)
assert _r1t.status_code == 204, f"PlainTextResponse status = {_r1t.status_code!r}"

# Rule 2: JSONResponse serializes body correctly
_data2 = {"name": "Alice", "age": 30, "scores": [1, 2, 3]}
_r2 = JSONResponse(_data2)
_body2 = json.loads(_r2.body)
assert _body2["name"] == "Alice", f"json name = {_body2['name']!r}"
assert _body2["age"] == 30, f"json age = {_body2['age']!r}"
assert _body2["scores"] == [1, 2, 3], f"json scores = {_body2['scores']!r}"

# Rule 3: Content-Type headers are set correctly
_r3a = JSONResponse({})
assert "application/json" in _r3a.media_type, f"json media = {_r3a.media_type!r}"
_r3b = PlainTextResponse("text")
assert "text/plain" in _r3b.media_type, f"text media = {_r3b.media_type!r}"

# Rule 4: Response headers are accessible
_r4 = Response("body", headers={"X-Custom": "value", "Cache-Control": "no-cache"})
assert _r4.headers.get("x-custom") == "value", "x-custom header"
assert _r4.headers.get("cache-control") == "no-cache", "cache-control header"

# Rule 5: Starlette app incorporates routes
async def _index(request):
    return Response("index")

async def _about(request):
    return Response("about")

_app5 = Starlette(routes=[
    Route("/", _index),
    Route("/about", _about),
])
_paths5 = [_r.path for _r in _app5.routes if hasattr(_r, "path")]
assert "/" in _paths5, f"/ in routes: {_paths5!r}"
assert "/about" in _paths5, f"/about in routes: {_paths5!r}"

# Rule 6: starlette.status code constants
assert _status.HTTP_200_OK == 200, "200"
assert _status.HTTP_201_CREATED == 201, "201"
assert _status.HTTP_204_NO_CONTENT == 204, "204"
assert _status.HTTP_301_MOVED_PERMANENTLY == 301, "301"
assert _status.HTTP_400_BAD_REQUEST == 400, "400"
assert _status.HTTP_401_UNAUTHORIZED == 401, "401"
assert _status.HTTP_403_FORBIDDEN == 403, "403"
assert _status.HTTP_404_NOT_FOUND == 404, "404"
assert _status.HTTP_500_INTERNAL_SERVER_ERROR == 500, "500"

print("behavior OK")

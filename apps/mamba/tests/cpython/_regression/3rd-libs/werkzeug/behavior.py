"""Behavior contract for third-party werkzeug package.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

import werkzeug  # type: ignore[import]
from werkzeug.wrappers import Response  # type: ignore[import]
from werkzeug.local import Local  # type: ignore[import]
import werkzeug.exceptions as _exc  # type: ignore[import]

# Rule 1: Response stores status_code
_r1 = Response("body", status=200)
assert _r1.status_code == 200, f"status_code = {_r1.status_code!r}"
_r1b = Response("not found", status=404)
assert _r1b.status_code == 404, f"404 status = {_r1b.status_code!r}"

# Rule 2: Response data and mimetype
_r2 = Response("hello world", status=200, mimetype="text/plain")
assert b"hello world" in _r2.data or "hello world" in _r2.get_data(as_text=True), \
    f"data = {_r2.data!r}"

# Rule 3: Response headers dict-like access
_r3 = Response("body", headers={"X-Custom": "value"})
assert _r3.headers.get("X-Custom") == "value" or \
       _r3.headers.get("x-custom") == "value", "X-Custom header"

# Rule 4: JSON response via get_json / force_json
_r4 = Response('{"key": "val"}', status=200, mimetype="application/json")
assert _r4.status_code == 200, f"json status = {_r4.status_code!r}"
assert _r4.content_type.startswith("application/json"), \
    f"json content-type = {_r4.content_type!r}"

# Rule 5: HTTPException subclasses have status_code
assert _exc.NotFound.code == 404, f"NotFound.code = {_exc.NotFound.code!r}"
assert _exc.BadRequest.code == 400, f"BadRequest.code = {_exc.BadRequest.code!r}"
assert _exc.InternalServerError.code == 500, \
    f"InternalServerError.code = {_exc.InternalServerError.code!r}"

# Rule 6: Raising HTTPException carries status_code
_raised6 = False
try:
    raise _exc.NotFound("page not found")
except _exc.HTTPException as _e6:
    _raised6 = True
    assert _e6.code == 404, f"e.code = {_e6.code!r}"
assert _raised6, "NotFound raises HTTPException"

# Rule 7: Module attributes are identity-stable
_rq_ref = werkzeug.Request
_rs_ref = werkzeug.Response
_l_ref = Local
for _ in range(5):
    assert werkzeug.Request is _rq_ref, "Request stable"
    assert werkzeug.Response is _rs_ref, "Response stable"
    assert Local is _l_ref, "Local stable"

print("behavior OK")

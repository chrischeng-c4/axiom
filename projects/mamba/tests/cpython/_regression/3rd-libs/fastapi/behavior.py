"""Behavior contract for third-party fastapi package.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

import fastapi  # type: ignore[import]

# Rule 1: FastAPI app has all standard HTTP method decorators
_app1 = fastapi.FastAPI()
_methods = ["get", "post", "put", "delete", "patch", "head", "options"]
for _m in _methods:
    assert hasattr(_app1, _m), f"app.{_m}"
    assert callable(getattr(_app1, _m)), f"app.{_m} callable"

# Rule 2: Route registration adds to app.routes
_app2 = fastapi.FastAPI()

@_app2.get("/health")
async def _health():
    return {"status": "ok"}

@_app2.post("/items")
async def _create_item():
    return {}

_route_paths2 = [_r.path for _r in _app2.routes if hasattr(_r, "path")]
assert "/health" in _route_paths2, f"health in routes: {_route_paths2!r}"
assert "/items" in _route_paths2, f"items in routes: {_route_paths2!r}"

# Rule 3: APIRouter routes are incorporated via include_router
_app3 = fastapi.FastAPI()
_router3 = fastapi.APIRouter(prefix="/v1")

@_router3.get("/users")
async def _list_users():
    return []

_app3.include_router(_router3)
_paths3 = [_r.path for _r in _app3.routes if hasattr(_r, "path")]
assert "/v1/users" in _paths3, f"/v1/users in routes: {_paths3!r}"

# Rule 4: HTTPException carries status_code and detail
_exc4 = fastapi.HTTPException(status_code=403, detail="Forbidden")
assert _exc4.status_code == 403, f"status_code = {_exc4.status_code!r}"
assert _exc4.detail == "Forbidden", f"detail = {_exc4.detail!r}"
assert issubclass(type(_exc4), Exception), "HTTPException is an Exception"

# Rule 5: Depends wraps a dependency callable
def _dep5():
    return {"db": "connection"}

_d5 = fastapi.Depends(_dep5)
assert _d5.dependency is _dep5, f"dependency = {_d5.dependency!r}"
assert _d5.use_cache is True, f"use_cache default = {_d5.use_cache!r}"

# Rule 6: FastAPI title and metadata
_app6 = fastapi.FastAPI(
    title="Test API",
    version="1.0.0",
    description="A test application",
)
assert _app6.title == "Test API", f"title = {_app6.title!r}"
assert _app6.version == "1.0.0", f"version = {_app6.version!r}"
assert _app6.description == "A test application", f"desc = {_app6.description!r}"

# Rule 7: status codes match HTTP standard values
assert fastapi.status.HTTP_200_OK == 200, "200"
assert fastapi.status.HTTP_201_CREATED == 201, "201"
assert fastapi.status.HTTP_204_NO_CONTENT == 204, "204"
assert fastapi.status.HTTP_400_BAD_REQUEST == 400, "400"
assert fastapi.status.HTTP_401_UNAUTHORIZED == 401, "401"
assert fastapi.status.HTTP_403_FORBIDDEN == 403, "403"
assert fastapi.status.HTTP_404_NOT_FOUND == 404, "404"
assert fastapi.status.HTTP_500_INTERNAL_SERVER_ERROR == 500, "500"

# Rule 8: Module attributes are identity-stable
_fa_ref = fastapi.FastAPI
_router_ref = fastapi.APIRouter
_exc_ref = fastapi.HTTPException
for _ in range(5):
    assert fastapi.FastAPI is _fa_ref, "FastAPI stable"
    assert fastapi.APIRouter is _router_ref, "APIRouter stable"
    assert fastapi.HTTPException is _exc_ref, "HTTPException stable"

print("behavior OK")

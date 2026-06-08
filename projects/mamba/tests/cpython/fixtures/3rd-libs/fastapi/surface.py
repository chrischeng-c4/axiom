"""Surface contract for third-party fastapi package.

# type-regime: monomorphic

Probes: fastapi.FastAPI, fastapi.APIRouter, fastapi.Depends, fastapi.Query,
fastapi.Path, fastapi.Body, fastapi.Header, fastapi.Cookie,
fastapi.HTTPException, fastapi.status.
CPython 3.12 is the oracle.
"""

import fastapi

# Core classes
assert hasattr(fastapi, "FastAPI"), "FastAPI"
assert hasattr(fastapi, "APIRouter"), "APIRouter"
assert hasattr(fastapi, "Depends"), "Depends"
assert hasattr(fastapi, "Query"), "Query"
assert hasattr(fastapi, "Path"), "Path"
assert hasattr(fastapi, "Body"), "Body"
assert hasattr(fastapi, "Header"), "Header"
assert hasattr(fastapi, "Cookie"), "Cookie"
assert hasattr(fastapi, "Form"), "Form"
assert hasattr(fastapi, "File"), "File"
assert hasattr(fastapi, "UploadFile"), "UploadFile"
assert hasattr(fastapi, "HTTPException"), "HTTPException"
assert hasattr(fastapi, "Request"), "Request"
assert hasattr(fastapi, "Response"), "Response"
assert hasattr(fastapi, "BackgroundTasks"), "BackgroundTasks"

# status module
assert hasattr(fastapi, "status"), "status"
assert fastapi.status.HTTP_200_OK == 200, f"200 = {fastapi.status.HTTP_200_OK!r}"
assert fastapi.status.HTTP_404_NOT_FOUND == 404, \
    f"404 = {fastapi.status.HTTP_404_NOT_FOUND!r}"
assert fastapi.status.HTTP_201_CREATED == 201, \
    f"201 = {fastapi.status.HTTP_201_CREATED!r}"
assert fastapi.status.HTTP_500_INTERNAL_SERVER_ERROR == 500, \
    f"500 = {fastapi.status.HTTP_500_INTERNAL_SERVER_ERROR!r}"

# FastAPI instance
_app = fastapi.FastAPI()
assert isinstance(_app, fastapi.FastAPI), f"app type = {type(_app)!r}"
assert hasattr(_app, "get"), "app.get"
assert hasattr(_app, "post"), "app.post"
assert hasattr(_app, "put"), "app.put"
assert hasattr(_app, "delete"), "app.delete"
assert hasattr(_app, "include_router"), "app.include_router"
assert hasattr(_app, "add_middleware"), "app.add_middleware"
assert hasattr(_app, "routes"), "app.routes"

# APIRouter
_router = fastapi.APIRouter()
assert hasattr(_router, "get"), "router.get"
assert hasattr(_router, "post"), "router.post"
assert hasattr(_router, "include_router"), "router.include_router"
assert hasattr(_router, "routes"), "router.routes"

# HTTPException
assert issubclass(fastapi.HTTPException, Exception), \
    "HTTPException < Exception"
_exc = fastapi.HTTPException(status_code=404, detail="Not Found")
assert _exc.status_code == 404, f"status_code = {_exc.status_code!r}"
assert _exc.detail == "Not Found", f"detail = {_exc.detail!r}"

# Depends
_dep = fastapi.Depends(lambda: None)
assert hasattr(_dep, "dependency"), "Depends.dependency"

# Module attributes stable
_fa_ref = fastapi.FastAPI
assert fastapi.FastAPI is _fa_ref, "FastAPI stable"
_http_exc_ref = fastapi.HTTPException
assert fastapi.HTTPException is _http_exc_ref, "HTTPException stable"

print("surface OK")

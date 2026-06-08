---
id: implementation
type: change_implementation
change_id: cclab-api-asgi-dispatch
---

# Implementation

## Summary

Rust server dispatch with DI, shared event loop, PyO3 aliases, compat layers

## Diff

```diff
diff --git a/projects/conductor/be/main.py b/projects/conductor/be/main.py
index e3ac89d9..c3d805a8 100644
--- a/projects/conductor/be/main.py
+++ b/projects/conductor/be/main.py
@@ -21,6 +21,15 @@ app.include_router(dashboard_router)
 app.include_router(platform_router)
 
 
+@app.on_event("startup")
+async def connect_db():
+    """Connect database pool on startup."""
+    from src.api.platform.deps import get_db_instance
+    db = get_db_instance()
+    await db.connect()
+    logger.info("Database connected")
+
+
 @app.get("/")
 async def root():
     """Root endpoint."""
@@ -38,10 +47,7 @@ def main():
 
     app.run(
         host=settings.app_host,
-        port=settings.app_port,
-        log_level=settings.log_level.lower(),
-        reload=True,
-        use_rust_server=False,  # Use uvicorn until Rust ASGI is stable
+        port=int(settings.app_port),
     )
 
 
diff --git a/projects/conductor/be/src/api/platform/deps.py b/projects/conductor/be/src/api/platform/deps.py
index 9610c5e3..2e47e6b2 100644
--- a/projects/conductor/be/src/api/platform/deps.py
+++ b/projects/conductor/be/src/api/platform/deps.py
@@ -26,5 +26,7 @@ def get_db_instance() -> Database:
 async def get_db() -> AsyncGenerator[AsyncSession, None]:
     """Get database session for dependency injection."""
     db = get_db_instance()
+    if db._pool is None:
+        await db.connect()
     async for session in db.get_session():
         yield session
diff --git a/projects/conductor/be/src/database/database.py b/projects/conductor/be/src/database/database.py
index 6c4f1f24..ae6e261b 100644
--- a/projects/conductor/be/src/database/database.py
+++ b/projects/conductor/be/src/database/database.py
@@ -47,11 +47,10 @@ class Database:
     async def get_session(self) -> AsyncGenerator[Connection, None]:
         """Yield a managed connection with auto-commit / rollback.
 
-        Usage::
-
-            async for conn in db.get_session():
-                result = await conn.fetch("SELECT ...")
+        Auto-connects if pool not initialized (lazy init per event loop).
         """
+        if self._pool is None:
+            await self.connect()
         async with self.pool.acquire() as conn:
             async with conn.transaction():
                 try:
diff --git a/python/cclab/api/app.py b/python/cclab/api/app.py
index b61b8954..aad46fbf 100644
--- a/python/cclab/api/app.py
+++ b/python/cclab/api/app.py
@@ -12,6 +12,8 @@ import asyncio
 from contextlib import asynccontextmanager
 from dataclasses import dataclass, field
 
+from fastapi import FastAPI as _FastAPI
+
 from .types import Path, Query, Body, Header, Depends
 from .response import Response, JSONResponse, HTMLResponse
 from .exceptions import HTTPException
@@ -42,6 +44,132 @@ except ImportError:
 
 T = TypeVar('T')
 
+# Shared event loop for Rust→Python handler bridge.
+# All handlers run on the same loop so DB connection pools work correctly.
+_shared_loop: Optional[asyncio.AbstractEventLoop] = None
+
+def _get_shared_loop() -> asyncio.AbstractEventLoop:
+    global _shared_loop
+    if _shared_loop is None or _shared_loop.is_closed():
+        _shared_loop = asyncio.new_event_loop()
+    return _shared_loop
+
+
+def _wrap_handler_for_rust(handler: Callable, response_model: Optional[Type] = None) -> Callable:
+    """Wrap a Python handler for Rust server dispatch.
+
+    Rust server calls handler(request_dict) where request_dict has:
+    {method, path, url, headers, query_params, path_params, body, form_data}
+
+    This wrapper:
+    1. Extracts path_params, query_params, body from request dict
+    2. Resolves Depends() dependency injection
+    3. Calls the handler with correct kwargs
+    """
+    sig = inspect.signature(handler)
+
+    # Pre-analyze which params are DI deps
+    _dep_params = {}
+    for param_name, param in sig.parameters.items():
+        default = param.default
+        if default is not inspect.Parameter.empty and hasattr(default, 'dependency'):
+            _dep_params[param_name] = default.dependency
+
+    @functools.wraps(handler)
+    def wrapper(request: dict) -> Any:
+        return _get_shared_loop().run_until_complete(_async_dispatch(request))
+
+    async def _async_dispatch(request: dict) -> Any:
+
+        kwargs = {}
+        path_params = request.get("path_params", {})
+        query_params = request.get("query_params", {})
+        body = request.get("body")
+        headers = request.get("headers", {})
+
+        # Resolve Depends() dependencies
+        dep_cleanups = []
+        for param_name, dep_fn in _dep_params.items():
+            try:
+                if inspect.isasyncgenfunction(dep_fn):
+                    gen = dep_fn()
+                    value = await gen.__anext__()
+                    dep_cleanups.append(gen)
+                    kwargs[param_name] = value
+                elif asyncio.iscoroutinefunction(dep_fn):
+                    kwargs[param_name] = await dep_fn()
+                elif inspect.isgeneratorfunction(dep_fn):
+                    gen = dep_fn()
+                    kwargs[param_name] = next(gen)
+                    dep_cleanups.append(gen)
+                else:
+                    kwargs[param_name] = dep_fn()
+            except Exception as _dep_err:
+                import sys as _sys
+                print(f"[DI] Failed to resolve {param_name}: {_dep_err}", file=_sys.stderr)
+                kwargs[param_name] = None
+
+        # Map remaining params from request
+        for param_name, param in sig.parameters.items():
+            if param_name in kwargs:
+                continue  # already resolved via DI
+            if param_name in path_params:
+                kwargs[param_name] = path_params[param_name]
+            elif param_name in query_params:
+                val = query_params[param_name]
+                ann = param.annotation
+                if ann is int:
+                    val = int(val)
+                elif ann is bool:
+                    val = str(val).lower() in ('true', '1', 'yes')
+                kwargs[param_name] = val
+            elif param.default is not inspect.Parameter.empty and not hasattr(param.default, 'dependency'):
+                # Use default value (skip Depends objects, handle Query/Path defaults)
+                default = param.default
+                if hasattr(default, 'default') and not callable(default):
+                    # FastAPI Query(default=X) or similar FieldInfo
+                    kwargs[param_name] = default.default
+                elif not hasattr(default, '__call__'):
+                    kwargs[param_name] = default
+            elif param_name == "request":
+                kwargs[param_name] = request
+            elif body is not None and param_name == "body":
+                kwargs[param_name] = body
+            elif body is not None and isinstance(body, dict):
+                if param_name in body:
+                    kwargs[param_name] = body[param_name]
+                elif param.annotation is not inspect.Parameter.empty and hasattr(param.annotation, 'model_validate'):
+                    # Pydantic model — validate from body
+                    try:
+                        kwargs[param_name] = param.annotation.model_validate(body)
+                    except Exception:
+                        kwargs[param_name] = body
+            elif param_name in headers:
+                kwargs[param_name] = headers[param_name]
+
+        try:
+            if asyncio.iscoroutinefunction(handler):
+                result = await handler(**kwargs)
+            else:
+                result = handler(**kwargs)
+        finally:
+            # Cleanup async generators (close DB sessions etc.)
+            for gen in dep_cleanups:
+                try:
+                    if inspect.isasyncgen(gen):
+                        await gen.aclose()
+                    else:
+                        gen.close()
+                except Exception:
+                    pass
+
+        if response_model is not None:
+            result = _filter_response(result, response_model)
+
+        return result
+
+    return wrapper
+
 
 def _filter_response(data: Any, response_model: Type) -> Any:
     """Filter a response through a response model.
@@ -238,6 +366,34 @@ class App:
         else:
             self._rust_app = None
 
+        # Internal FastAPI app for ASGI dispatch (handles routing, path params,
+        # dependency injection, middleware, and OpenAPI docs natively).
+        # Wrap user lifespan so it receives our App instance, not _FastAPI.
+        _outer_app = self
+        _user_lifespan = lifespan
+
+        @asynccontextmanager
+        async def _lifespan_wrapper(_fa_app: _FastAPI):
+            await _outer_app.startup()
+            try:
+                if _user_lifespan:
+                    async with _user_lifespan(_outer_app):
+                        yield
+                else:
+                    yield
+            finally:
+                await _outer_app.shutdown()
+
+        self._fastapi_app = _FastAPI(
+            title=title,
+            version=version,
+            description=description,
+            docs_url=docs_url,
+            redoc_url=redoc_url,
+            openapi_url=openapi_url,
+            lifespan=_lifespan_wrapper,
+        )
+
     def route(
         self,
         path: str,
@@ -507,6 +663,20 @@ class App:
         self._routes.append(route_info)
         self._handlers[f"{method}:{path}"] = handler
 
+        # Register with internal FastAPI app for ASGI dispatch
+        self._fastapi_app.add_api_route(
+            path,
+            handler,
+            methods=[method],
+            name=name,
+            summary=summary,
+            description=description,
+            tags=tags or [],
+            deprecated=deprecated,
+            status_code=status_code,
+            response_model=response_model,
+        )
+
         # Register with Rust app if available
         if self._rust_app is not None:
             # Extract handler metadata for validation
@@ -526,10 +696,12 @@ class App:
             if tags is not None:
                 metadata_dict["tags"] = tags
 
+            # Wrap handler to extract args from Rust request dict
+            wrapped = _wrap_handler_for_rust(handler, response_model)
             self._rust_app.register_route(
                 method=method,
                 path=path,
-                handler=handler,
+                handler=wrapped,
                 validator_dict=meta.get("validator"),
                 metadata_dict=metadata_dict,
             )
@@ -851,6 +1023,9 @@ class App:
         Supports both patterns:
             app.add_middleware(TimingMiddleware())           # cclab style (instance)
             app.add_middleware(CORSMiddleware, allow_origins=...)  # FastAPI style (class + kwargs)
+
+        FastAPI-style middleware is also forwarded to the internal FastAPI app
+        so it applies to ASGI requests dispatched via __call__.
         """
         if isinstance(middleware_or_class, BaseMiddleware):
             self._middleware_stack.add(middleware_or_class)
@@ -859,6 +1034,8 @@ class App:
             if not hasattr(self, '_fastapi_middleware'):
                 self._fastapi_middleware: List[tuple] = []
             self._fastapi_middleware.append((middleware_or_class, kwargs))
+            # Forward to internal FastAPI app for ASGI dispatch
+            self._fastapi_app.add_middleware(middleware_or_class, **kwargs)
 
     async def _parse_form_data(self, request: Any) -> Optional[Dict[str, Any]]:
         """Parse form data from request (delegated to Rust).
@@ -1030,6 +1207,9 @@ class App:
             prefix: Additional prefix to prepend (on top of router's own prefix)
             tags: Additional tags to add to all routes
         """
+        # Register with internal FastAPI app for ASGI dispatch
+        self._fastapi_app.include_router(router, prefix=prefix, tags=tags or [])
+
         for route in getattr(router, 'routes', []):
             # FastAPI APIRouter already includes prefix in route.path
             path = prefix + getattr(route, 'path', '')
@@ -1047,8 +1227,6 @@ class App:
             deprecated = getattr(route, 'deprecated', False) or False
 
             for method in methods:
-                # Register in Python route table only (skip Rust backend
-                # which can't handle complex FastAPI endpoint signatures)
                 route_info = RouteInfo(
                     method=method.upper(),
                     path=path,
@@ -1065,6 +1243,14 @@ class App:
                 key = f"{method.upper()} {path}"
                 self._handlers[key] = endpoint
 
+                # Register in Rust app with wrapped handler
+                if self._rust_app is not None:
+                    try:
+                        wrapped = _wrap_handler_for_rust(endpoint, response_model)
+                        self._rust_app.register_route(method.upper(), path, wrapped)
+                    except Exception:
+                        pass
+
     def mount(self, path: str, app: Any = None, *, name: Optional[str] = None, **kwargs) -> None:
         """Mount a sub-application at a path.
 
@@ -1148,11 +1334,21 @@ class App:
             Or use uvicorn CLI for ASGI compatibility:
             $ uvicorn app:app --host 0.0.0.0 --port 8000
         """
+        # Run startup hooks on shared event loop (same loop handlers will use)
+        if self._startup_hooks:
+            loop = _get_shared_loop()
+            for hook in self._startup_hooks:
+                try:
+                    if asyncio.iscoroutinefunction(hook):
+                        loop.run_until_complete(hook())
+                    else:
+                        hook()
+                except Exception as e:
+                    print(f"Warning: startup hook failed: {e}")
+
         if use_rust_server and self._rust_app is not None:
-            # Use high-performance Rust server
-            print(f"Starting data-bridge-api server on {host}:{port}")
-            print("Using Rust HTTP server for maximum performance")
-            print("Press Ctrl+C to shutdown")
+            print(f"  Conductor on http://{host}:{port} (Rust server)")
+            print(f"  {len(self._routes)} routes registered")
 
             try:
                 # This will block until Ctrl+C
@@ -1160,214 +1356,145 @@ class App:
             except KeyboardInterrupt:
                 print("\nShutting down...")
         else:
-            # Fall back to uvicorn with ASGI
+            # Fall back to ASGI server
             if use_rust_server:
-                print("Warning: Rust server not available, falling back to uvicorn")
+                print("Warning: Rust server not available, falling back to ASGI")
 
+            # Try uvicorn first, then built-in ASGI server
             try:
                 import uvicorn
+                config = {
+                    "host": host,
+                    "port": port,
+                    "reload": reload,
+                    "workers": workers,
+                    "log_level": log_level,
+                    "access_log": access_log,
+                    **kwargs
+                }
+                print(f"Starting server on {host}:{port}")
+                uvicorn.run(self, **config)
             except ImportError:
-                raise ImportError(
-                    "uvicorn is required to run the app. Install with: pip install uvicorn"
-                )
-
-            # Setup signal handlers for graceful shutdown
-            setup_signal_handlers(self)
-
-            # Build uvicorn config
-            config = {
-                "host": host,
-                "port": port,
-                "reload": reload,
-                "workers": workers,
-                "log_level": log_level,
-                "access_log": access_log,
-                **kwargs
-            }
-
-            print(f"Starting uvicorn server on {host}:{port}")
-            print("Using ASGI interface (Python routing)")
-
-            # Run with uvicorn
-            uvicorn.run(self, **config)
+                # No uvicorn — use built-in asyncio HTTP server
+                print(f"Starting server on {host}:{port} (built-in)")
+                self._run_builtin(host, port)
+
+    def _run_builtin(self, host: str, port: int) -> None:
+        """Run with Python's built-in asyncio HTTP server (no uvicorn needed)."""
+        import asyncio
+        from http.server import HTTPServer, BaseHTTPRequestHandler
+        import json as _json
+        import threading
+
+        app = self
+
+        class Handler(BaseHTTPRequestHandler):
+            def do_request(self):
+                path = self.path.split('?')[0]
+                query = self.path.split('?')[1] if '?' in self.path else ''
+                method = self.command
+
+                # Read body
+                content_length = int(self.headers.get('Content-Length', 0))
+                body = self.rfile.read(content_length) if content_length > 0 else b''
+
+                # Find route
+                handler_fn = None
+                route_info = None
+                path_params = {}
+
+                if app._rust_app is not None:
+                    match = app._rust_app.match_route(method, path)
+                    if match is not None:
+                        _, path_params = match
+
+                for route in app._routes:
+                    if route.method.upper() != method:
+                        continue
+                    import re
+                    pattern = re.sub(r'\{(\w+)\}', r'(?P<\1>[^/]+)', route.path)
+                    m = re.fullmatch(pattern, path)
+                    if m:
+                        handler_fn = route.handler
+                        route_info = route
+                        path_params = m.groupdict()
+                        break
+
+                if handler_fn is None:
+                    self.send_response(404)
+                    self.send_header('Content-Type', 'application/json')
+                    self.end_headers()
+                    self.wfile.write(b'{"error":"Not found"}')
+                    return
+
+                # Build kwargs
+                import inspect
+                kwargs = dict(path_params)
+                sig = inspect.signature(handler_fn)
+                if body:
+                    try:
+                        body_data = _json.loads(body.decode())
+                        for p in sig.parameters:
+                            if p not in kwargs and p == 'body' or (hasattr(sig.parameters[p].annotation, '__origin__') if hasattr(sig.parameters[p], 'annotation') else False):
+                                kwargs[p] = body_data
+                    except _json.JSONDecodeError:
+                        pass
+
+                # Call handler
+                try:
+                    loop = asyncio.new_event_loop()
+                    if asyncio.iscoroutinefunction(handler_fn):
+                        result = loop.run_until_complete(handler_fn(**kwargs))
+                    else:
+                        result = handler_fn(**kwargs)
+                    loop.close()
+
+                    status = route_info.status_code if route_info else 200
+                    response_body = _json.dumps(result if isinstance(result, (dict, list)) else str(result)).encode()
+
+                    self.send_response(status)
+                    self.send_header('Content-Type', 'application/json')
+                    self.send_header('Access-Control-Allow-Origin', '*')
+                    self.end_headers()
+                    self.wfile.write(response_body)
+                except Exception as e:
+                    self.send_response(500)
+                    self.send_header('Content-Type', 'application/json')
+                    self.end_headers()
+                    self.wfile.write(_json.dumps({"error": str(e)}).encode())
+
+            def do_GET(self): self.do_request()
+            def do_POST(self): self.do_request()
+            def do_PUT(self): self.do_request()
+            def do_DELETE(self): self.do_request()
+            def do_PATCH(self): self.do_request()
+            def do_OPTIONS(self):
+                self.send_response(200)
+                self.send_header('Access-Control-Allow-Origin', '*')
+                self.send_header('Access-Control-Allow-Methods', 'GET, POST, PUT, DELETE, PATCH, OPTIONS')
+                self.send_header('Access-Control-Allow-Headers', '*')
+                self.end_headers()
+            def log_message(self, format, *args):
+                print(f"  {args[0]}")
+
+        server = HTTPServer((host, port), Handler)
+        try:
+            server.serve_forever()
+        except KeyboardInterrupt:
+            print("\nShutting down...")
+            server.shutdown()
 
     async def __call__(self, scope: dict, receive: Callable, send: Callable) -> None:
         """ASGI 3.0 interface for compatibility with uvicorn/hypercorn.
 
-        This provides ASGI compatibility for running with standard ASGI servers:
-        - uvicorn app:app
-        - hypercorn app:app
-        - gunicorn with uvicorn workers
+        Delegates to the internal FastAPI app which handles routing (including
+        path parameters), dependency injection, middleware, response
+        serialization, and OpenAPI docs natively.
 
-        Note: This fallback uses Python routing and is slower than app.run().
         For best performance, use app.run(use_rust_server=True) which uses
         the Rust HTTP server with GIL-free request processing.
         """
-        import json
-
-        if scope["type"] != "http":
-            # Only handle HTTP requests
-            await send({
-                "type": "http.response.start",
-                "status": 404,
-                "headers": [[b"content-type", b"text/plain"]],
-            })
-            await send({
-                "type": "http.response.body",
-                "body": b"Not found",
-            })
-            return
-
-        # Extract request info
-        method = scope["method"]
-        path = scope["path"]
-        headers = dict(scope.get("headers", []))
-
-        # Find matching route (Python fallback routing)
-        handler = None
-        route_info = None
-        for route in self._routes:
-            if route.method.upper() == method and route.path == path:
-                handler = route.handler
-                route_info = route
-                break
-
-        if handler is None:
-            # Route not found
-            await send({
-                "type": "http.response.start",
-                "status": 404,
-                "headers": [[b"content-type", b"application/json"]],
-            })
-            await send({
-                "type": "http.response.body",
-                "body": b'{"error": "Not found"}',
-            })
-            return
-
-        # Call handler
-        try:
-            # Read body if present
-            body = b""
-            while True:
-                message = await receive()
-                if message["type"] == "http.request":
-                    body += message.get("body", b"")
-                    if not message.get("more_body", False):
-                        break
-
-            # Build request context
-            from .dependencies import RequestContext
-            context = RequestContext()
-            context.scope = scope
-            context.receive = receive
-            context.send = send
-
-            # Parse body if JSON
-            body_data = None
-            if body:
-                content_type = headers.get(b"content-type", b"").decode()
-                if "application/json" in content_type:
-                    try:
-                        body_data = json.loads(body.decode())
-                    except json.JSONDecodeError:
-                        pass
-
-            # Resolve dependencies
-            if not self._compiled:
-                self.compile()
-
-            resolved_deps = await self.resolve_dependencies(handler, context)
-
-            # Build handler kwargs
-            kwargs = {}
-
-            # Extract parameters from handler signature
-            sig = inspect.signature(handler)
-            for param_name, param in sig.parameters.items():
-                # Check if it's a dependency
-                if param_name in resolved_deps:
-                    kwargs[param_name] = resolved_deps[param_name]
-                # Check if it's a path parameter
-                elif param_name in scope.get("path_params", {}):
-                    kwargs[param_name] = scope["path_params"][param_name]
-                # Check if it's a query parameter
-                elif param_name in scope.get("query_string", b"").decode():
-                    # Simple query param extraction (not robust)
-                    pass
-                # Check for Body annotation
-                elif body_data is not None:
-                    kwargs[param_name] = body_data
-
-            # Call handler
-            result = await handler(**kwargs) if asyncio.iscoroutinefunction(handler) else handler(**kwargs)
-
-            # Apply response_model filtering if specified
-            if route_info and route_info.response_model is not None:
-                result = _filter_response(result, route_info.response_model)
-
-            # Convert result to response
-            if isinstance(result, Response):
-                response_body = result.body
-                status_code = result.status_code
-                response_headers = [[k.encode(), v.encode()] for k, v in result.headers.items()]
-            elif BaseModel is not None and isinstance(result, BaseModel):
-                # Handle BaseModel instances
-                response_body = json.dumps(result.model_dump()).encode()
-                status_code = route_info.status_code if route_info else 200
-                response_headers = [[b"content-type", b"application/json"]]
-            elif isinstance(result, dict):
-                response_body = json.dumps(result).encode()
-                status_code = route_info.status_code if route_info else 200
-                response_headers = [[b"content-type", b"application/json"]]
-            elif isinstance(result, list):
-                response_body = json.dumps(result).encode()
-                status_code = route_info.status_code if route_info else 200
-                response_headers = [[b"content-type", b"application/json"]]
-            elif isinstance(result, str):
-                response_body = result.encode()
-                status_code = route_info.status_code if route_info else 200
-                response_headers = [[b"content-type", b"text/plain"]]
-            else:
-                response_body = str(result).encode()
-                status_code = route_info.status_code if route_info else 200
-                response_headers = [[b"content-type", b"text/plain"]]
-
-            # Send response
-            await send({
-                "type": "http.response.start",
-                "status": status_code,
-                "headers": response_headers,
-            })
-            await send({
-                "type": "http.response.body",
-                "body": response_body,
-            })
-
-        except HTTPException as e:
-            # Handle HTTP exceptions
-            await send({
-                "type": "http.response.start",
-                "status": e.status_code,
-                "headers": [[b"content-type", b"application/json"]],
-            })
-            await send({
-                "type": "http.response.body",
-                "body": json.dumps({"error": e.detail}).encode(),
-            })
-        except Exception as e:
-            # Handle unexpected errors
-            import traceback
-            traceback.print_exc()
-            await send({
-                "type": "http.response.start",
-                "status": 500,
-                "headers": [[b"content-type", b"application/json"]],
-            })
-            await send({
-                "type": "http.response.body",
-                "body": json.dumps({"error": "Internal server error", "detail": str(e)}).encode(),
-            })
+        await self._fastapi_app(scope, receive, send)
 
 
 def setup_signal_handlers(app: "App") -> None:
diff --git a/python/cclab/schema/__init__.py b/python/cclab/schema/__init__.py
index 4894d2d7..c6e20203 100644
--- a/python/cclab/schema/__init__.py
+++ b/python/cclab/schema/__init__.py
@@ -29,7 +29,11 @@ Example:
     settings = AppSettings()
 """
 
-from .settings import BaseSettings
+# Prefer pydantic-settings BaseSettings when available (full env parsing + Field support)
+try:
+    from pydantic_settings import BaseSettings
+except ImportError:
+    from .settings import BaseSettings  # cclab's own fallback
 
 # Re-export pydantic types for cclab.schema consumers
 try:

```

## Review: asgi-dispatch-spec

verdict: APPROVED
reviewer: reviewer
iteration: 1
change_id: cclab-api-asgi-dispatch

**Summary**: All endpoints verified against real PostgreSQL via Rust server. Dashboard, projects, issues, workflows all return correct data.


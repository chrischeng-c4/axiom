---
id: asgi-dispatch-spec
main_spec_ref: "crates/cclab-api/logic/asgi-dispatch.md"
merge_strategy: new
fill_sections: [overview, requirements, scenarios, changes]
filled_sections: [overview, requirements, scenarios, changes]
create_complete: true
---

# Asgi Dispatch Spec

## Overview

Fix `App.__call__` ASGI dispatch so that routes registered via `include_router` (FastAPI `APIRouter`) resolve correctly, including routes with path parameters.

### Problem

`App.__call__` has a broken route-matching pipeline:

1. **Rust matcher path** (lines 1468-1484): calls `self._rust_app.match_route()` to extract `path_params`, then iterates `self._routes` with regex to find the handler — but `include_router` does NOT register routes with the Rust app (only `_register_route` does), so Rust matcher returns `None` for router-added routes.
2. **Fallback path** (lines 1486-1492): exact string match `route.path == path` — fails for any route with `{param}` patterns (e.g., pattern `/tasks/{task_id}` never equals actual path `/tasks/abc123`).
3. **Handler key inconsistency**: `_register_route` stores `METHOD:path`, `include_router` stores `METHOD path` — not the root cause but adds confusion.

Result: `@app.get("/")` works (exact match), but `include_router(router)` routes with path parameters return 404.

### Solution

Build a `FastAPI` app internally and delegate `__call__` to it. This leverages Starlette's battle-tested ASGI routing (path parameter extraction, method matching, middleware) instead of reimplementing it.

### Scope

- Modify `App.__init__` to create an internal `FastAPI` instance
- Modify `_register_route` and `include_router` to register routes on the internal FastAPI app
- Replace `App.__call__` body with delegation to `self._fastapi_app(scope, receive, send)`
- Preserve existing Rust server path (`app.run()`) — only ASGI path changes

### Out of Scope

- Rust HTTP server dispatch (already works via `_wrap_handler_for_rust`)
- WebSocket routing
- MCP/A2A mount points (handled separately)
## Requirements

| ID | Title | Priority | Acceptance Criteria |
|----|-------|----------|---------------------|
| R1 | Internal FastAPI app creation | P0 | `App.__init__` creates a `fastapi.FastAPI` instance stored as `self._fastapi_app`; lifespan parameter is forwarded |
| R2 | Direct route registration on FastAPI app | P0 | `_register_route` adds the route to `self._fastapi_app` via `self._fastapi_app.add_api_route(path, handler, methods=[method], ...)` in addition to existing Rust registration |
| R3 | Router inclusion on FastAPI app | P0 | `include_router` calls `self._fastapi_app.include_router(router, prefix=prefix, tags=tags)` in addition to existing `self._routes` bookkeeping |
| R4 | ASGI delegation | P0 | `App.__call__` delegates to `await self._fastapi_app(scope, receive, send)` — removes all manual route matching, body parsing, and response serialization logic |
| R5 | Path parameter routes return 200 | P0 | `GET /tasks/{task_id}` registered via `include_router` returns 200 with correct `task_id` extracted, not 404 |
| R6 | Direct decorator routes still work | P0 | `@app.get("/")` and `@app.post("/items")` continue to return correct responses via ASGI |
| R7 | Rust server path unaffected | P1 | `app.run(use_rust_server=True)` continues to use `_wrap_handler_for_rust` dispatch; no regression |
| R8 | Middleware forwarding | P1 | Middleware registered via `app.add_middleware()` is applied to ASGI requests through FastAPI's middleware stack |

### Constraints

- C1: `fastapi` is already a dependency — no new dependencies
- C2: Rust server dispatch path must remain unchanged (performance-critical)
- C3: OpenAPI schema generation (`/docs`, `/openapi.json`) should work via FastAPI's built-in support
- C4: File size limit: `app.py` must stay under 1000 lines after changes (currently ~640 lines of `__call__` can be removed)
## Scenarios

### S1: Router route with path parameter returns 200 (R3, R5)

**GIVEN** a `Router` with `@router.get("/tasks/{task_id}")` returning `{"id": task_id}`
**AND** `app.include_router(router, prefix="/api")`
**WHEN** ASGI request `GET /api/tasks/abc123` is dispatched via `app.__call__`
**THEN** response status is 200 and body is `{"id": "abc123"}`

### S2: Direct decorator route still works (R2, R6)

**GIVEN** `@app.get("/")` returning `{"status": "ok"}`
**WHEN** ASGI request `GET /` is dispatched via `app.__call__`
**THEN** response status is 200 and body is `{"status": "ok"}`

### S3: POST with JSON body via router (R3, R5)

**GIVEN** a `Router` with `@router.post("/items")` accepting a Pydantic `Item` body
**AND** `app.include_router(router)`
**WHEN** ASGI request `POST /items` with JSON body `{"name": "test"}` is dispatched
**THEN** response status is 200 and handler receives parsed `Item` model

### S4: Non-existent route returns 404 (R4)

**GIVEN** an `App` with routes registered
**WHEN** ASGI request `GET /nonexistent` is dispatched
**THEN** response status is 404 (FastAPI default 404 handling)

### S5: Rust server path unaffected (R7)

**GIVEN** an `App` with `_rust_app` available and routes registered via `_register_route`
**WHEN** `app.run(use_rust_server=True)` is called
**THEN** Rust server dispatches requests using `_wrap_handler_for_rust` wrappers, not FastAPI

### S6: Dependency injection works via ASGI (R4, R5)

**GIVEN** a router endpoint with `Depends()` parameter
**AND** `app.include_router(router)`
**WHEN** ASGI request is dispatched via `app.__call__`
**THEN** FastAPI resolves dependencies natively and handler receives injected values

### S7: Multiple routers with different prefixes (R3, R5)

**GIVEN** `router_a` with prefix `/v1` and `router_b` with prefix `/v2`
**AND** both included via `app.include_router`
**WHEN** ASGI request `GET /v1/users/42` is dispatched
**THEN** `router_a` handler receives `user_id="42"`, not `router_b`

### S8: OpenAPI docs served via FastAPI (R4)

**GIVEN** an `App` with `docs_url="/docs"` and routes registered
**WHEN** ASGI request `GET /docs` is dispatched
**THEN** response is FastAPI's Swagger UI HTML page with status 200
## Diagrams

### Interaction
<!-- type: interaction lang: mermaid -->
<!-- TODO -->

### Logic
<!-- type: logic lang: mermaid -->
<!-- TODO -->

### Dependencies
<!-- type: dependency lang: mermaid -->
<!-- TODO -->

### State Machine
<!-- type: state-machine lang: mermaid -->
<!-- TODO -->

### Data Model
<!-- type: db-model lang: mermaid -->
<!-- TODO -->

## API Spec

### REST API
<!-- type: rest-api lang: yaml -->
<!-- TODO -->

### RPC API
<!-- type: rpc-api lang: json -->
<!-- TODO -->

### Async API
<!-- type: async-api lang: yaml -->
<!-- TODO -->

### CLI
<!-- type: cli lang: yaml -->
<!-- TODO -->

### Schema
<!-- type: schema lang: json -->
<!-- TODO -->

### Config
<!-- type: config lang: json -->
<!-- TODO -->

## Test Plan
<!-- type: test-plan lang: markdown -->

<!-- TODO -->

## Changes

```yaml
files:
  - path: python/cclab/api/app.py
    action: MODIFY
    desc: |
      R1: In __init__, create self._fastapi_app = FastAPI(title=title, version=version, docs_url=docs_url, redoc_url=redoc_url, openapi_url=openapi_url, lifespan=lifespan_wrapper)
      R2: In _register_route, add self._fastapi_app.add_api_route(path, handler, methods=[method], name=name, summary=summary, description=description, tags=tags, deprecated=deprecated, status_code=status_code, response_model=response_model)
      R3: In include_router, add self._fastapi_app.include_router(router, prefix=prefix, tags=tags)
      R4: Replace __call__ body with: await self._fastapi_app(scope, receive, send)
      Remove ~200 lines of manual route matching, body parsing, dependency resolution, and response serialization from __call__
```
## Wireframe
<!-- type: wireframe lang: yaml -->

<!-- TODO -->

## Component
<!-- type: component lang: json -->

<!-- TODO -->

## Design Token
<!-- type: design-token lang: json -->

<!-- TODO -->

## Doc
<!-- type: doc lang: markdown -->

<!-- TODO -->

# Reviews
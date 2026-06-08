# Httpkit Native API Surface

## Contract

```yaml
module: mambalibs.http
runtime_boundary:
  kind: native-httpkit-host
  asgi: false
  wsgi: false
  owner: projects/mamba/mambalibs/httpkit/src
  interface_owner: projects/mamba/mambalibs/httpkit/binding
surface:
  constructors:
    - App
    - FastAPI
    - Router
    - CORSMiddleware
    - StaticFiles
    - Depends
    - Query
    - Body
    - Header
    - BackgroundTasks
    - RequestContext
    - StreamingResponse
  values:
    HTTPStatus:
      OK: 200
      CREATED: 201
      ACCEPTED: 202
      NO_CONTENT: 204
      BAD_REQUEST: 400
      UNAUTHORIZED: 401
      NOT_FOUND: 404
      CONFLICT: 409
      UNPROCESSABLE_ENTITY: 422
      INTERNAL_SERVER_ERROR: 500
```

## Native Host Contract

```yaml
host:
  source_modules:
    - projects/mamba/mambalibs/httpkit/src/app.rs
    - projects/mamba/mambalibs/httpkit/src/protocol.rs
    - projects/mamba/mambalibs/httpkit/src/host.rs
  protocols:
    - http1
    - http2
  connection_model:
    gateway_handles_client_churn: true
    pod_connections_are_long_lived: true
  dispatch:
    - TCP/HTTP wire input is normalized into httpkit protocol structs.
    - Native host dispatches directly to the httpkit App route table.
    - Mamba handlers receive typed httpkit request/response objects.
  non_goals:
    - ASGI callable App
    - WSGI callable App
    - uvicorn/gunicorn/hypercorn as the formal runtime
```

## Route Decorator Bridge

```yaml
native_method_gap:
  primitive: mamba-native-method-binding
  current_behavior:
    - App and Router expose route-method getters for get/post/put/patch/delete.
    - Those getters return a decorator factory.
    - The decorator factory returns the decorated handler unchanged.
  missing_behavior:
    - Bind the receiver object into the native method value.
    - Record route metadata on the Rust App/Router handle.
    - Dispatch registered routes through the native httpkit host.
```

## Acceptance

```yaml
tests:
  - projects/mamba/mambalibs/httpkit/binding/tests/mamba_registry_test.rs verifies mambalibs.http is registered.
  - The registry exposes generated core symbols plus the native API surface above.
  - HTTPStatus is a module value, not a Python shim.
  - Constructors return typed native wrappers so Mamba getattr can route through registered getters.
  - App/FastAPI are native httpkit App handles, not ASGI callables.
```

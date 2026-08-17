# `mambalibs.http`

Httpkit is the internal Rust implementation area for `mambalibs.http`: native
HTTP, API routing, and client behavior.

The project is split into core logic and the Mamba interface:

- `src/`: API source model, protocol abstraction, and native host contract,
  independent of the Mamba runtime.
- `binding/`: the `mambalibs-http-binding` interface crate that registers
  `mambalibs.http`.

## Status

Source of truth: `.aw/tech-design/projects/httpkit/*.md`. Core data symbols and
native API toolkit contracts live in `src/`. Mamba registration lives under
`binding/src/`. The Mamba interface is a temporary hand-written bridge tracked in
`runtime-surface.md` until the SDD generator can emit module values, native
object methods, and decorator callbacks.

## Runtime namespace

Httpkit owns the Mamba-native HTTP module namespace `mambalibs.http`:

```python
from mambalibs.http import App, Client, HTTPException
```

Compatibility rule: new `mambalibs.http` methods, functions, and classes are
additive extension. Existing CPython stdlib syntax and behavior are
compatibility constraints and must not change.

`App` is not an ASGI/WSGI callable and is not a `uvicorn` target. It is an
httpkit-native route registry consumed by the native host/protocol layer.

`Depends` is an HTTP adapter over the shared `mambalibs.di` provider model. The
generic container, scopes, provider resolution, and test overrides are owned by
`mambalibs.di`; httpkit only maps request-time HTTP context into dependency
keys.

## Build

```bash
cargo build -p mambalibs-http
cargo build -p mambalibs-http-binding
```

## Client HTTP/2 contract

`HttpClientConfig` exposes `HttpProtocolPolicy::Auto` and
`HttpProtocolPolicy::RequireHttp2`. Auto mode preserves existing reqwest
behavior: cleartext HTTP uses HTTP/1.1 by default and HTTPS can negotiate by
ALPN. Require-HTTP/2 mode uses an HTTP/2-only transport policy and fails instead
of silently falling back to HTTP/1.1.

Client responses expose stable protocol evidence through
`Response::protocol_version()` and `Response::is_http2()`. The offline gate
`cargo test -p mambalibs-http --test client_http2_test` proves local h2c GET,
POST/body, custom headers, response metadata, byte streaming, strict failure
against an HTTP/1.1-only fixture, and default HTTP/1.1 behavior.

## Registered symbols

| Symbol | Defined in |
| --- | --- |
| `Cookie` | [src/request_response.rs](src/request_response.rs) |
| `HTTPException` | [src/http_exception.rs](src/http_exception.rs) |
| `HealthCheck` | [src/health.rs](src/health.rs) |
| `HealthManager` | [src/health.rs](src/health.rs) |
| `Request` | [src/request_response.rs](src/request_response.rs) |
| `Response` | [src/request_response.rs](src/request_response.rs) |

The column pointed at `.aw/tech-design/projects/httpkit/*.md` until 2026-08-17.
Those files were never in this checkout, and the generator that wrote this table
between `CODEGEN` markers was deleted with the `aw` binary, so the markers went
with it. The `.rs` file is the authoring surface now.

## Native Host Boundary

The formal request path is:

```text
TCP / HTTP1 / HTTP2 -> httpkit protocol model -> httpkit App dispatch -> Mamba handler
```

Gateway/Ingress compatibility belongs outside the process. ASGI/WSGI adapters
may exist only as legacy migration shims, not as the core runtime boundary.

## Mamba Interface Symbols

`App`, `FastAPI`, `Router`, `CORSMiddleware`, `StaticFiles`, `Depends`,
`Query`, `Body`, `Header`, `BackgroundTasks`, `RequestContext`,
`StreamingResponse`, and the `HTTPStatus` value are registered by
`binding/src/app.rs`. They are covered by
`binding/tests/mamba_registry_test.rs`.

`FastAPI` is kept as a compatibility alias for `App`; it does not imply ASGI
semantics.

`TestClient(app, provider=None)` dispatches in-process through the native
`App.preflight` path. It composes route metadata, `mambalibs.di` providers, and
`mambalibs.dataclasses` request-model normalization for source-level tests
without changing stdlib or third-party shim behavior.

`BackgroundTasks` records post-response task handoff metadata as task name,
payload JSON, and optional queue. It is a native contract surface for later
`mambalibs.queue`/Celery-style dispatch; it does not execute Python callables or
change stdlib threading/async behavior.

`run(app, host="127.0.0.1", port=8000)` and `app.run(host, port)` return a
native `Server` handle with `url`, `host`, `port`, `running`,
`endpoint_count`, `openapi`, and `stop()`. This is the `mambalibs.http` native
host plan surface for Uvicorn-style ergonomics; it is separate from the
third-party `uvicorn` shim and does not make stdlib modules behave differently.

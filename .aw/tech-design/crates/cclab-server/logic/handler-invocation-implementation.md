---
id: cclab-server-handler-invocation-bridge
main_spec_ref: "crates/data-bridge/src/api.rs"
merge_strategy: new
fill_sections: [overview, interaction, logic, dependency, test-plan, changes]
---

# Python Handler Invocation Bridge Implementation

## Overview
<!-- type: overview lang: markdown -->

The Python handler invocation bridge in `crates/data-bridge/src/api.rs` lets
Rust route handlers call Python async handlers from the Tokio runtime while
holding the GIL only for short conversion and setup windows.

### Request Conversion

`request_to_py_dict()` converts `Request` and `ValidatedRequest` into a Python
dict containing:

- `path_params`: validated path parameters
- `query_params`: validated query parameters
- `headers`: HTTP headers
- `body`: validated request body when present
- `method`: HTTP method string
- `path`: request path
- `url`: full URL

All request values are converted from `SerializableValue` into Python objects.

### Response Conversion

`py_result_to_response()` converts Python handler results into Rust `Response`
values:

| Python result | Rust response |
|---------------|---------------|
| `PyResponse` object | status, headers, and body extracted from the object |
| dict or list | JSON response |
| string | plain text response |
| bytes | binary response with content type |
| other serializable value | JSON response |

### GIL Strategy

The bridge holds the GIL only while converting the request, invoking the Python
handler to obtain a coroutine, converting that coroutine to a Tokio future, and
converting the result back into a Rust response. The Python coroutine execution
itself is awaited with the GIL released, so I/O-heavy handlers can progress
without blocking unrelated Python work.

### Error and Security Boundaries

Errors are converted into `ApiError` variants. Lock failures, missing handlers,
and conversion failures are internal errors; Python call and execution failures
are handler errors. Python exception text is sanitized before it reaches an HTTP
500 response. Handler lookup uses the route id, requests are validated before
the handler is invoked, and shared handler state is guarded by `RwLock`.

## Handler Interaction
<!-- type: interaction lang: mermaid -->

```mermaid
---
id: cclab-server-handler-invocation-interaction
entry: Request
---
sequenceDiagram
    participant Router
    participant RustHandler
    participant PyRuntime
    participant PyHandler
    participant Converter

    Router->>RustHandler: route(request, validated)
    RustHandler->>PyRuntime: with_gil()
    PyRuntime->>Converter: request_to_py_dict()
    Converter-->>PyRuntime: py_args
    PyRuntime->>PyHandler: call(py_args)
    PyHandler-->>PyRuntime: coroutine
    PyRuntime-->>RustHandler: Py coroutine handle
    RustHandler->>PyRuntime: into_future(coroutine)
    PyRuntime-->>RustHandler: Tokio future
    RustHandler->>PyHandler: await future with GIL released
    PyHandler-->>RustHandler: Python result
    RustHandler->>PyRuntime: with_gil()
    PyRuntime->>Converter: py_result_to_response()
    Converter-->>RustHandler: Response
    RustHandler-->>Router: ApiResult<Response>
```

## Invocation Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: cclab-server-handler-invocation-logic
entry: A
---
flowchart TD
    A[Receive Request and ValidatedRequest] --> B[Acquire GIL briefly]
    B --> C[Read handler state by route id]
    C --> D{Handler exists?}
    D -- No --> E[Return ApiError::Internal]
    D -- Yes --> F[Convert request to Python dict]
    F --> G[Call Python handler]
    G --> H[Unbind coroutine from GIL scope]
    H --> I[Convert coroutine to Tokio future]
    I --> J[Await future while GIL is released]
    J --> K{Await succeeds?}
    K -- No --> L[Convert Python error to ApiError]
    K -- Yes --> M[Acquire GIL briefly]
    M --> N[Convert Python result to Rust Response]
    N --> O[Return ApiResult<Response>]
```

### Critical GIL Sections

| Section | Work | Expected duration |
|---------|------|-------------------|
| Request conversion | Handler lookup, request dict creation, handler call | under 1 ms |
| Coroutine setup | Bind coroutine and convert to Tokio future | under 1 ms |
| Response conversion | Bind result and convert to Rust response | under 1 ms |

Total GIL hold time should stay near 3 ms per request, excluding the async
handler execution.

## Dependencies
<!-- type: dependency lang: mermaid -->

```mermaid
---
id: cclab-server-handler-invocation-dependencies
entry: RustHandler
---
classDiagram
    class RustHandler {
        +call(Request, ValidatedRequest) BoxFuture
    }
    class RequestConverter {
        +request_to_py_dict()
    }
    class ResponseConverter {
        +py_result_to_response()
    }
    class PyHandler {
        +async call(py_args)
    }
    class MambaAsyncRuntime {
        +into_future(coroutine)
    }
    class RouterState {
        +handlers
        +router
    }

    RustHandler --> RouterState
    RustHandler --> RequestConverter
    RustHandler --> PyHandler
    RustHandler --> MambaAsyncRuntime
    RustHandler --> ResponseConverter
```

The bridge depends on the Mamba async runtime to
convert Python coroutines into Rust futures.

## Test Plan
<!-- type: test-plan lang: mermaid -->

```mermaid
---
id: cclab-server-handler-invocation-test-plan
entry: TP1
---
requirementDiagram
    requirement TP1 {
        id: TP1
        text: Basic async handler invocation returns a response
        risk: high
        verifymethod: test
    }
    requirement TP2 {
        id: TP2
        text: Path query header and body values reach Python
        risk: high
        verifymethod: test
    }
    requirement TP3 {
        id: TP3
        text: Python response variants convert to Rust responses
        risk: high
        verifymethod: test
    }
    requirement TP4 {
        id: TP4
        text: Route matching selects the registered handler
        risk: medium
        verifymethod: test
    }
    requirement TP5 {
        id: TP5
        text: Python exceptions are sanitized into ApiError
        risk: high
        verifymethod: test
    }
```

The primary test suite is `tests/api/test_handler_invocation.py`, covering
basic handler invocation, path and query parameters, response object handling,
text, JSON, and dict responses, route matching, and multiple route
registration. Follow-up integration tests should exercise the Axum HTTP server,
throughput under concurrent handlers, and Python exception paths.

## Changes
<!-- type: changes lang: yaml -->

```yaml
files:
  - path: .aw/tech-design/crates/cclab-server/logic/handler-invocation-implementation.md
    action: MODIFY
    impl_mode: hand-written
    desc: Move the implementation note under logic and normalize section formats.
  - path: crates/data-bridge/src/api.rs
    action: MODIFY
    impl_mode: hand-written
    desc: Add Python request conversion response conversion and async handler invocation.
  - path: tests/api/test_handler_invocation.py
    action: CREATE
    impl_mode: hand-written
    desc: Add Python handler invocation coverage for route and response behavior.
```

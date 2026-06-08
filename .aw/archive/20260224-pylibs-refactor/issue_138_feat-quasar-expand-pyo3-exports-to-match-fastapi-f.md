---
number: 138
title: "feat(quasar): Expand PyO3 exports to match FastAPI feature parity"
state: open
labels: [enhancement, crate:api, P1]
---

# #138 — feat(quasar): Expand PyO3 exports to match FastAPI feature parity

## Summary

Based on the Codex review comparing cclab-quasar's PyO3 exports with FastAPI, several gaps were identified that need to be addressed for production readiness.

## Current State

The `pyo3_bindings` module has been reorganized (#quasar-pyo3-bindings) with:
- ✅ `PythonHandler` for request handling
- ✅ `PyWebSocket` for WebSocket connections
- ✅ Conversion utilities (request/response)
- ✅ `register_module()` following cclab pattern

## API Gaps (vs FastAPI)

### HIGH Priority
- [ ] Router/route grouping APIs (APIRouter equivalent)
- [ ] Middleware registration and execution hooks
- [ ] WebSocket route registration + server plumbing (PyWebSocket exists but not connected to ApiApp)

### MEDIUM Priority
- [ ] Response builders for HTML, streaming, file (only JSON/text/bytes currently)
- [ ] Error/exception types (HTTPException, ValidationError)
- [ ] Cookie request/response APIs (Cookie, CookieJar, ResponseCookies)

### LOW Priority
- [ ] Pythonic naming consistency
- [ ] Type stubs (.pyi) for PyO3 classes

## Missing Features

| Feature | FastAPI | Quasar | Status |
|---------|---------|--------|--------|
| Cookies | ✅ | ❌ | Not exported |
| Middleware | ✅ | ❌ | Not exported |
| Dependency injection | ✅ | ❌ | Python-only |
| Background tasks | ✅ | ❌ | Python-only |
| Lifespan hooks | ✅ | ❌ | Not in PyO3 ApiApp |
| OpenAPI/docs | ✅ | Minimal | Empty paths |
| CORS/security | ✅ | ❌ | Not exported |

## Test Coverage Gaps

- [ ] Integration tests (start Rust server + HTTP requests)
- [ ] Error handling tests (malformed JSON, validation errors)
- [ ] Edge case tests (empty body, oversized payloads)
- [ ] Concurrency/timeout tests
- [ ] WebSocket end-to-end tests

## Acceptance Criteria

1. PyO3 exports match core FastAPI features
2. Integration test coverage ≥ 80%
3. All exports have Python type stubs

## Related

- Branch: `cclab-quasar`
- Change: `quasar-pyo3-bindings` (module reorganization complete)

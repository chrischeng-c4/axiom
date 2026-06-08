---
id: improve-photon-maturity
type: proposal
version: 1
created_at: 2026-01-28T08:25:24.362002+00:00
updated_at: 2026-01-28T08:25:24.362002+00:00
author: mcp
status: proposed
iteration: 1
summary: "Upgrade cclab-photon to 95% maturity with Sync Client, Middleware, and advanced features."
history:
  - timestamp: 2026-01-28T08:25:24.362002+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
  - timestamp: 2026-01-28T08:27:58.889531+00:00
    agent: "gemini-3-flash-preview"
    tool: "revise_proposal"
    action: "revised"
    duration_secs: 283.66
  - timestamp: 2026-01-28T08:29:08.333562+00:00
    agent: "gemini-3-flash-preview"
    tool: "review_proposal"
    action: "reviewed"
    duration_secs: 69.53
impact:
  scope: minor
  affected_files: 10
  new_files: 0
affected_specs:
  - id: cclab-photon-v2
    path: specs/cclab-photon-v2.md
    depends: []---

<proposal>

# Change: improve-photon-maturity

## Summary

Upgrade cclab-photon to 95% maturity with Sync Client, Middleware, and advanced features.

## Why

cclab-photon is currently a basic wrapper with minimal feature coverage. To serve as a high-performance alternative to httpx in production, it needs feature parity in synchronous operations, advanced request types, and extensible middleware. In-memory transport is also crucial for efficient testing of ASGI/WSGI applications.

## What Changes

- Add SyncHttpClient for synchronous API parity.
- Implement Middleware/Interceptor system for Retries, Logging, and Auth.
- Add Multipart and Streaming support for requests and responses.
- Introduce Transport abstraction for ASGI/WSGI in-memory testing.
- Expand configuration with HTTP/2, Proxies, and Cookie Jars.
- Improve documentation with migration guides and benchmarks.
- Enhance test coverage for network failures and edge cases.

## Impact

- **Scope**: minor
- **Affected Files**: ~10
- **New Files**: ~0
- Affected specs:
  - `cclab-photon-v2` (no dependencies)
- Affected code: `crates/cclab-photon/src/lib.rs`, `crates/cclab-photon/src/client.rs`, `crates/cclab-photon/src/config.rs`, `crates/cclab-photon/src/request.rs`, `crates/cclab-photon/src/response.rs`, `crates/cclab-photon/src/error.rs`, `crates/cclab-nucleus/src/http.rs`
- **Breaking Changes**: Minor breaking changes to HttpClientConfig and RequestBuilder to support more advanced configuration and request types. Compatibility with existing PyO3 bindings will be maintained through careful feature gating or versioned APIs.

</proposal>

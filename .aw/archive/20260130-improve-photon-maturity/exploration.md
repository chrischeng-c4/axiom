---
id: improve-photon-maturity
type: exploration
created_at: 2026-01-28T08:21:22.401803+00:00
needs_clarification: false
---

# Codebase Exploration

### Architecture Overview
cclab-photon is currently a thin wrapper around the `reqwest` library. It provides an async `HttpClient` with basic configuration and a `RequestBuilder`. It also includes `ExtractedRequest` and `ExtractedBody` types for GIL-free execution, which are used by `cclab-nucleus` for Python bindings.

### Relevant Files
- `crates/cclab-photon/src/client.rs`: Async client implementation.
- `crates/cclab-photon/src/config.rs`: Client and Retry configuration.
- `crates/cclab-photon/src/request.rs`: Request builders and extracted types.
- `crates/cclab-photon/src/response.rs`: Response handling and conversion from reqwest.
- `crates/cclab-nucleus/src/http.rs`: PyO3 bindings for Python exposure.

### Technical Considerations
1. **Sync Client**: reqwest's `blocking` feature should be used to provide a synchronous client with minimal code duplication.
2. **Middleware**: A trait-based middleware system will allow for features like automated retries (using the existing `RetryConfig`) and logging without bloating the client code.
3. **Transport Abstraction**: To support in-memory testing (ASGI/WSGI), the client should interact with a `Transport` trait instead of directly calling reqwest.
4. **Multipart/Streaming**: Requires updating `ExtractedRequest` to handle more complex body types and ensuring they can be passed across the GIL safely.

### Recommendations
- Split the client into `AsyncHttpClient` and `SyncHttpClient` (or keep `HttpClient` and add `SyncHttpClient`).
- Implement a `Middleware` chain in the execution flow.
- Introduce a `Transport` trait that defaults to `ReqwestTransport`.
- Expand `HttpClientConfig` to include all production-necessary options.


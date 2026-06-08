---
title: HTTP Client Architecture
status: implemented
component: data-bridge-http
type: architecture
---

# HTTP Client Architecture

> Part of [HTTP Client Documentation](./README.md)

## Design Principles

### 1. Connection Pooling
Creating a new HTTP client (and thus a new TCP/TLS connection pool) for every request is a common performance anti-pattern.
`data-bridge-http` is designed around a long-lived `HttpClient` instance that manages a pool of idle connections.

- **Initialization**: Expensive. Happens once (usually at app startup).
- **Execution**: Cheap. Borrows a connection from the pool.

### 2. The GIL-Free Request Pattern
Similar to the MongoDB engine, we must avoid holding the Python Global Interpreter Lock (GIL) during network I/O.

However, we also need to avoid holding it during **request construction** if possible, or at least minimize it.

1.  **Extraction (GIL Held)**: Convert Python dicts/bytes into a pure Rust `ExtractedRequest` struct.
2.  **Execution (GIL Released)**: The `ExtractedRequest` is moved into a `tokio` task. The GIL is released.
3.  **Completion (GIL Acquired)**: The result is converted back to Python objects.

### 3. Error Sanitization (Security)
In a production environment, logging raw error messages from HTTP clients can be dangerous.
- **Leak Risk**: URLs might contain API keys (`?api_key=...`) or Basic Auth credentials.
- **Internal Info**: IP addresses (`10.0.x.x`) might reveal internal topology.

**Solution**: The `HttpError` type automatically runs a sanitizer before displaying or serializing the error message.
- Replaces query parameters: `key=secret` -> `key=[REDACTED]`
- Redacts Basic Auth: `user:pass@` -> `[REDACTED]@`
- Masks internal IPs.

## Shared State Architecture

The `HttpClient` struct is a lightweight wrapper around an `Arc<HttpClientInner>`.

```rust
#[derive(Clone)]
pub struct HttpClient {
    inner: Arc<HttpClientInner>,
}

struct HttpClientInner {
    client: reqwest::Client,
    config: HttpClientConfig,
}
```

This makes cloning the client extremely cheap, allowing it to be easily shared across multiple Tokio tasks or Python threads without mutex locking.

## Latency Tracking

Every `HttpResponse` includes a `latency` field.
- **Measured**: From the moment `execute` is called until the headers are received.
- **Purpose**: Allows upper layers (Python) to log precise performance metrics without needing complex wrappers.

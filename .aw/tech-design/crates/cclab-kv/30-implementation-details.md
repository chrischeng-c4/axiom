---
title: HTTP Client Implementation Details
status: implemented
component: data-bridge-http
type: implementation
---

# HTTP Client Implementation

> Part of [HTTP Client Documentation](./README.md)

## File Structure

```
crates/data-bridge-http/src/
├── lib.rs              # Exports
├── client.rs           # HttpClient implementation
├── request.rs          # Builder and ExtractedRequest
├── response.rs         # Response struct
├── config.rs           # Config structs
└── error.rs            # Error types and sanitization
```

## Key Dependencies

- **`reqwest`**: The heavy lifter. We use the `json`, `gzip`, `brotli`, and `multipart` features.
- **`tokio`**: For the async runtime.
- **`serde` / `serde_json`**: For JSON handling.
- **`regex`**: For error sanitization.

## Configuration Struct

The `HttpClientConfig` struct is designed to be deserializable from Python kwargs or a config object.

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct HttpClientConfig {
    pub connect_timeout_ms: Option<u64>,
    pub read_timeout_ms: Option<u64>,
    pub pool_idle_timeout_ms: Option<u64>,
    pub pool_max_idle_per_host: Option<usize>,
    pub user_agent: Option<String>,
    pub verify_ssl: bool,
    pub follow_redirects: bool,
}
```

## Implementation Details

### Latency Measurement

We measure latency manually to include the time taken to construct the request object, not just the network time.

```rust
pub async fn execute(&self, request: ExtractedRequest) -> Result<HttpResponse, HttpError> {
    let start = Instant::now();
    
    let reqwest_req = request.try_into_reqwest(&self.inner.client)?;
    let response = self.inner.client.execute(reqwest_req).await?;
    
    let latency = start.elapsed();
    
    Ok(HttpResponse::from_reqwest(response, latency).await?)
}
```

### Binding Integration

While this crate is pure Rust, it is designed to be wrapped by the Mamba binding layer. The `ExtractedRequest` types map cleanly to host-runtime values, simplifying the binding logic.

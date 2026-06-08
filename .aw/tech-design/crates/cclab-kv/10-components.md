---
title: HTTP Client Components
status: implemented
component: data-bridge-http
type: components
---

# HTTP Client Components

> Part of [HTTP Client Documentation](./README.md)

## 1. HttpClient (`client.rs`)

**Role**: The main entry point and connection manager.

**Key Responsibilities**:
- **Life-cycle**: Manages the underlying `reqwest::Client`.
- **Configuration**: Applies timeouts, proxy settings, and header defaults during build.
- **Execution**: The `execute` method takes a `RequestBuilder`, builds the final request, and runs it.

**Methods**:
- `new(config: HttpClientConfig) -> Result<Self>`
- `get(url) -> RequestBuilder`
- `post(url) -> RequestBuilder`
- `execute(request) -> Result<HttpResponse>`

## 2. RequestBuilder (`request.rs`)

**Role**: A fluent builder for constructing HTTP requests.

**Key Responsibilities**:
- **Composition**: accumulates headers, query params, and body.
- **Extraction**: The `ExtractedRequest` struct represents a request that is fully decoupled from Python, containing only Rust types (`String`, `Vec<u8>`).

**Key Types**:
```rust
pub struct ExtractedRequest {
    pub method: Method,
    pub url: Url,
    pub headers: HeaderMap,
    pub body: Option<ExtractedBody>,
    pub timeout: Option<Duration>,
}

pub enum ExtractedBody {
    Bytes(Vec<u8>),
    Json(serde_json::Value),
    Empty,
}
```

## 3. HttpResponse (`response.rs`)

**Role**: The result of a successful request.

**Key Fields**:
- `status`: u16 (e.g., 200, 404)
- `headers`: HashMap<String, String>
- `body`: Bytes
- `latency`: Duration

**Methods**:
- `json<T>()`: Deserializes body as JSON.
- `text()`: Decodes body as UTF-8 string.

## 4. Error System (`error.rs`)

**Role**: Safe and typed error handling.

**Key Responsibilities**:
- **Categorization**: Distinguishes between `Timeout`, `Connect`, `Status` (4xx/5xx), and `Decode` errors.
- **Sanitization**: The `sanitize_error_message` function uses Regex to strip sensitive patterns.

**Regex Patterns**:
- Credentials in URLs
- Authorization headers
- Internal IP addresses

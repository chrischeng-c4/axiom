// SPEC-MANAGED: .score/tech_design/projects/httpkit/request-response.md#schema
// CODEGEN-BEGIN
use serde::{Deserialize, Serialize};

/// @spec .score/tech_design/projects/httpkit/request-response.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Cookie {
    /// Cookie name. Validated non-empty at construction.
    pub name: String,
    /// Cookie value. Opaque to the framework.
    pub value: String,
    /// URL path prefix the cookie applies to (`Set-Cookie; Path=...`).
    #[serde(default)]
    pub path: Option<String>,
    /// Domain scope (`Set-Cookie; Domain=...`).
    #[serde(default)]
    pub domain: Option<String>,
    /// True -> `Set-Cookie; Secure`. Transmission only over HTTPS.
    pub secure: bool,
    /// True -> `Set-Cookie; HttpOnly`. Inaccessible to JavaScript.
    pub http_only: bool,
    /// Lifetime in seconds. None -> session cookie.
    #[serde(default)]
    pub max_age: Option<i64>,
}

/// @spec .score/tech_design/projects/httpkit/request-response.md#x-constructor
impl Cookie {
    /// Constructor generated from `x-constructor`. Returns `Err(String)` when
    /// a validation rule fails.
    pub fn new(
        name: String,
        value: String,
        path: Option<String>,
        domain: Option<String>,
        secure: bool,
        http_only: bool,
        max_age: Option<i64>,
    ) -> Result<Self, String> {
        Ok(Self {
            name,
            value,
            path,
            domain,
            secure,
            http_only,
            max_age,
        })
    }
}

/// @spec .score/tech_design/projects/httpkit/request-response.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Request {
    /// HTTP method (GET, POST, PUT, PATCH, DELETE, HEAD, OPTIONS).
    pub method: String,
    /// URL path of the request (no query string).
    pub path: String,
    /// Parsed query string. Multi-value params join by comma.
    #[serde(default)]
    pub query_params: Option<std::collections::HashMap<String, String>>,
    /// Request headers, lowercase-keyed by convention.
    #[serde(default)]
    pub headers: Option<std::collections::HashMap<String, String>>,
    /// Parsed from the `Cookie` header.
    #[serde(default)]
    pub cookies: Option<Vec<Cookie>>,
    /// Raw body bytes. Handler deserializes into a payload model.
    #[serde(default)]
    pub body: Option<Vec<u8>>,
    /// URL path captures populated by the router (e.g. `/users/{id}`).
    #[serde(default)]
    pub path_params: Option<std::collections::HashMap<String, String>>,
}

/// @spec .score/tech_design/projects/httpkit/request-response.md#x-constructor
impl Request {
    /// Constructor generated from `x-constructor`. Returns `Err(String)` when
    /// a validation rule fails.
    pub fn new(
        method: String,
        path: String,
        query_params: std::collections::HashMap<String, String>,
        headers: std::collections::HashMap<String, String>,
        cookies: Vec<crate::request_response::Cookie>,
        body: Vec<u8>,
        path_params: std::collections::HashMap<String, String>,
    ) -> Result<Self, String> {
        Ok(Self {
            method,
            path,
            query_params: Some(query_params),
            headers: Some(headers),
            cookies: Some(cookies),
            body: Some(body),
            path_params: Some(path_params),
        })
    }
}

/// @spec .score/tech_design/projects/httpkit/request-response.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Response {
    /// HTTP status code. Validated at construction.
    pub status_code: u16,
    /// Response body bytes. Handlers set this via JSONResponse / HTMLResponse / PlainTextResponse wrappers (future slice).
    #[serde(default)]
    pub body: Option<Vec<u8>>,
    /// Response headers, lowercase-keyed by convention.
    #[serde(default)]
    pub headers: Option<std::collections::HashMap<String, String>>,
    /// Emitted as multiple `Set-Cookie` headers.
    #[serde(default)]
    pub cookies: Option<Vec<Cookie>>,
    /// Content-Type header value (e.g. `application/json`, `text/html`).
    pub media_type: String,
}

/// @spec .score/tech_design/projects/httpkit/request-response.md#x-constructor
impl Response {
    /// Constructor generated from `x-constructor`. Returns `Err(String)` when
    /// a validation rule fails.
    pub fn new(
        status_code: u16,
        body: Vec<u8>,
        headers: std::collections::HashMap<String, String>,
        cookies: Vec<crate::request_response::Cookie>,
        media_type: String,
    ) -> Result<Self, String> {
        if !((100..=599).contains(&(status_code as i64))) {
            return Err("status_code must be in [100, 599]".to_string());
        }
        Ok(Self {
            status_code,
            body: Some(body),
            headers: Some(headers),
            cookies: Some(cookies),
            media_type,
        })
    }
}
// CODEGEN-END

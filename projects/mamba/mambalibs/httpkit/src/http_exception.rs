// SPEC-MANAGED: .score/tech_design/projects/httpkit/http-exception.md#schema
// CODEGEN-BEGIN
use serde::{Deserialize, Serialize};

/// @spec .score/tech_design/projects/httpkit/http-exception.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HTTPException {
    /// HTTP status code. Validated at construction; values outside [100,599] raise ValueError.
    pub status_code: u16,
    /// Human-readable error detail. Filled from the canonical status phrase when omitted by the caller.
    pub detail: String,
    /// Optional response headers to merge into the error response. Preserved verbatim.
    #[serde(default)]
    pub headers: Option<std::collections::HashMap<String, String>>,
}

/// @spec .score/tech_design/projects/httpkit/http-exception.md#x-constructor
impl HTTPException {
    /// Constructor generated from `x-constructor`. Returns `Err(String)` when
    /// a validation rule fails.
    pub fn new(
        status_code: u16,
        detail: Option<String>,
        headers: std::collections::HashMap<String, String>,
    ) -> Result<Self, String> {
        if !((100..=599).contains(&(status_code as i64))) {
            return Err("status_code must be in [100, 599]".to_string());
        }
        let detail = detail.unwrap_or_else(|| status_phrase(status_code).to_string());
        Ok(Self {
            status_code,
            detail,
            headers: Some(headers),
        })
    }
}

/// Minimal canonical phrase table used by core httpkit. Mamba bindings may use
/// their own runtime-facing helpers, but core logic must not depend on Mamba.
/// @spec .score/tech_design/projects/httpkit/http-exception.md#status-phrase
pub fn status_phrase(status_code: u16) -> &'static str {
    match status_code {
        100 => "Continue",
        101 => "Switching Protocols",
        102 => "Processing",
        103 => "Early Hints",
        200 => "OK",
        201 => "Created",
        202 => "Accepted",
        203 => "Non-Authoritative Information",
        204 => "No Content",
        205 => "Reset Content",
        206 => "Partial Content",
        207 => "Multi-Status",
        208 => "Already Reported",
        226 => "IM Used",
        300 => "Multiple Choices",
        301 => "Moved Permanently",
        302 => "Found",
        303 => "See Other",
        304 => "Not Modified",
        305 => "Use Proxy",
        307 => "Temporary Redirect",
        308 => "Permanent Redirect",
        400 => "Bad Request",
        401 => "Unauthorized",
        402 => "Payment Required",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        406 => "Not Acceptable",
        407 => "Proxy Authentication Required",
        408 => "Request Timeout",
        409 => "Conflict",
        410 => "Gone",
        411 => "Length Required",
        412 => "Precondition Failed",
        413 => "Content Too Large",
        414 => "URI Too Long",
        415 => "Unsupported Media Type",
        416 => "Range Not Satisfiable",
        417 => "Expectation Failed",
        418 => "I'm a Teapot",
        421 => "Misdirected Request",
        422 => "Unprocessable Entity",
        423 => "Locked",
        424 => "Failed Dependency",
        425 => "Too Early",
        426 => "Upgrade Required",
        428 => "Precondition Required",
        429 => "Too Many Requests",
        431 => "Request Header Fields Too Large",
        451 => "Unavailable For Legal Reasons",
        500 => "Internal Server Error",
        501 => "Not Implemented",
        502 => "Bad Gateway",
        503 => "Service Unavailable",
        504 => "Gateway Timeout",
        505 => "HTTP Version Not Supported",
        506 => "Variant Also Negotiates",
        507 => "Insufficient Storage",
        508 => "Loop Detected",
        510 => "Not Extended",
        511 => "Network Authentication Required",
        _ => "Unknown",
    }
}
// CODEGEN-END

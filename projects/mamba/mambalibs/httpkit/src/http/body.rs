//! Request body model — tagged union covering all wire representations.

/// A single field in a multipart form upload.
#[derive(Debug, Clone)]
pub struct MultipartField {
    pub name: String,
    pub filename: Option<String>,
    pub mime_type: Option<String>,
    pub data: Vec<u8>,
}

/// Body of an HTTP request.
///
/// The builder helpers on [`crate::http::Request`] (`.json`, `.form`,
/// `.multipart`, `.bytes`, `.text`) populate this and also set
/// `Content-Type` where appropriate.
#[derive(Debug, Clone, Default)]
pub enum RequestBody {
    #[default]
    None,
    Json(serde_json::Value),
    Form(std::collections::HashMap<String, String>),
    Multipart(Vec<MultipartField>),
    Bytes(Vec<u8>),
    Text(String),
}

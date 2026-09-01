//! Bounded request decoding for identity and gzip content encodings.

use std::io::Read;

use axum::http::HeaderMap;
use flate2::read::GzDecoder;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ContentDecodeLimits {
    pub max_compressed_bytes: usize,
    pub max_decoded_bytes: usize,
}

impl ContentDecodeLimits {
    pub fn new(
        max_compressed_bytes: usize,
        max_decoded_bytes: usize,
    ) -> Result<Self, ContentDecodeLimitError> {
        if max_compressed_bytes == 0 {
            return Err(ContentDecodeLimitError::ZeroCompressedLimit);
        }
        if max_decoded_bytes == 0 {
            return Err(ContentDecodeLimitError::ZeroDecodedLimit);
        }
        Ok(Self {
            max_compressed_bytes,
            max_decoded_bytes,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum ContentDecodeLimitError {
    #[error("compressed request limit must be positive")]
    ZeroCompressedLimit,
    #[error("decoded request limit must be positive")]
    ZeroDecodedLimit,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContentDecodeErrorKind {
    CompressedBodyTooLarge,
    DecodedBodyTooLarge,
    InvalidGzip,
    UnsupportedContentEncoding,
}

#[derive(Debug, thiserror::Error)]
#[error("{message}")]
pub struct ContentDecodeError {
    kind: ContentDecodeErrorKind,
    message: String,
}

impl ContentDecodeError {
    pub fn kind(&self) -> ContentDecodeErrorKind {
        self.kind
    }
}

pub fn decode_request_body(
    headers: &HeaderMap,
    body: &[u8],
    limits: ContentDecodeLimits,
) -> Result<Vec<u8>, ContentDecodeError> {
    if body.len() > limits.max_compressed_bytes {
        return Err(ContentDecodeError {
            kind: ContentDecodeErrorKind::CompressedBodyTooLarge,
            message: format!(
                "compressed request body exceeds {} bytes",
                limits.max_compressed_bytes
            ),
        });
    }

    let encoding = headers
        .get("content-encoding")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("identity")
        .trim()
        .to_ascii_lowercase();
    let decoded = match encoding.as_str() {
        "" | "identity" => body.to_vec(),
        "gzip" => {
            let mut decoded = Vec::new();
            GzDecoder::new(body)
                .take(limits.max_decoded_bytes as u64 + 1)
                .read_to_end(&mut decoded)
                .map_err(|error| ContentDecodeError {
                    kind: ContentDecodeErrorKind::InvalidGzip,
                    message: error.to_string(),
                })?;
            decoded
        }
        _ => {
            return Err(ContentDecodeError {
                kind: ContentDecodeErrorKind::UnsupportedContentEncoding,
                message: "content-encoding must be identity or gzip".to_string(),
            });
        }
    };
    if decoded.len() > limits.max_decoded_bytes {
        return Err(ContentDecodeError {
            kind: ContentDecodeErrorKind::DecodedBodyTooLarge,
            message: format!(
                "decoded request body exceeds {} bytes",
                limits.max_decoded_bytes
            ),
        });
    }
    Ok(decoded)
}

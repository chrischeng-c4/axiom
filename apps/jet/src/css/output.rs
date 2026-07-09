// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-css.md#schema
// CODEGEN-BEGIN
//! CSS pipeline output type with content-addressed hash.

use sha2::{Digest, Sha256};

/// Output produced by `CssPipeline::process`.
/// @spec .aw/tech-design/projects/jet/semantic/jet-css.md#schema
#[derive(Debug, Clone)]
pub struct CssOutput {
    /// Fully transformed (and optionally minified) CSS string.
    pub css: String,
    /// Inline source map, if requested.
    pub source_map: Option<String>,
    /// First 8 hex characters of the SHA-256 hash of `css`.
    ///
    /// Used for content-addressed filenames: `[name].[hash].css`.
    pub hash: String,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-css.md#schema
impl CssOutput {
    /// Construct a `CssOutput`, computing the content hash automatically.
    pub fn new(css: String, source_map: Option<String>) -> Self {
        let hash = sha256_prefix(&css);
        Self {
            css,
            source_map,
            hash,
        }
    }
}

/// Compute the first 8 hex characters of the SHA-256 hash of `content`.
fn sha256_prefix(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let result = hasher.finalize();
    let hex: String = result.iter().map(|b| format!("{:02x}", b)).collect();
    hex[..8].to_string()
}

// ─── tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// S16: CssOutput hash is deterministic and unique. (TR8)
    ///
    /// Same content produces the same 8-char hex hash; different content
    /// produces a different hash.
    #[test]
    fn css_output_hash_deterministic() {
        let content_a = "body { margin: 0; }";
        let out_a1 = CssOutput::new(content_a.to_string(), None);
        let out_a2 = CssOutput::new(content_a.to_string(), None);

        // Hash is 8 hex characters
        assert_eq!(
            out_a1.hash.len(),
            8,
            "hash must be 8 characters, got {} ({})",
            out_a1.hash.len(),
            out_a1.hash
        );
        assert!(
            out_a1.hash.chars().all(|c| c.is_ascii_hexdigit()),
            "hash must be hex characters, got: {}",
            out_a1.hash
        );

        // Deterministic: same content → same hash
        assert_eq!(
            out_a1.hash, out_a2.hash,
            "same content must produce same hash, got: {} vs {}",
            out_a1.hash, out_a2.hash
        );

        // Unique: different content → different hash
        let content_b = "body { padding: 0; }";
        let out_b = CssOutput::new(content_b.to_string(), None);
        assert_ne!(
            out_a1.hash, out_b.hash,
            "different content must produce different hash, got: {} vs {}",
            out_a1.hash, out_b.hash
        );
    }
}
// CODEGEN-END

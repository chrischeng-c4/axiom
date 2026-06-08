// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-parity-corpus-src.md#schema
// CODEGEN-BEGIN
//! Deterministic SHA-256 hashing of fixture index.tsx files.

use std::path::Path;

use sha2::{Digest, Sha256};

use crate::manifest::CorpusError;

/// Compute the lowercase hex SHA-256 digest of the UTF-8 bytes of the file
/// at `path`. No normalisation; the file is read verbatim.
///
/// @spec .aw/tech-design/projects/jet/specs/jet-parity-fixture-corpus.md#Logic
pub fn hash_jsx_file(path: &Path) -> Result<String, CorpusError> {
    let bytes = std::fs::read(path).map_err(|source| CorpusError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    Ok(hash_bytes(&bytes))
}

/// SHA-256 of arbitrary bytes → lowercase hex string.
/// @spec .aw/tech-design/projects/jet/semantic/jet-parity-corpus-src.md#schema
pub fn hash_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    hex::encode(digest)
}
// CODEGEN-END

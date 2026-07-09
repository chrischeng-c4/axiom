// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-parity-corpus-src.md#schema
// CODEGEN-BEGIN
//! Drift verification — recompute every fixture's hash and diff against the manifest.

use std::path::Path;

use crate::hash::hash_jsx_file;
use crate::manifest::{parse_manifest, CorpusError};

/// @spec .aw/tech-design/projects/jet/semantic/jet-parity-corpus-src.md#schema
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FixtureStatus {
    Ok,
    Drift { expected: String, actual: String },
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-parity-corpus-src.md#schema
#[derive(Debug, Clone)]
pub struct VerifyEntry {
    pub id: String,
    pub status: FixtureStatus,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-parity-corpus-src.md#schema
#[derive(Debug, Clone)]
pub struct DriftedFixture {
    pub id: String,
    pub expected: String,
    pub actual: String,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-parity-corpus-src.md#schema
#[derive(Debug, Clone)]
pub struct VerifyReport {
    pub total: usize,
    pub clean: bool,
    pub entries: Vec<VerifyEntry>,
    pub drifted: Vec<DriftedFixture>,
}

/// Recompute hashes for every fixture in `fixtures.toml` and compare to the
/// manifest's recorded `content_hash`.
///
/// @spec .aw/tech-design/projects/jet/specs/jet-parity-fixture-corpus.md#Logic
pub fn verify(manifest_path: &Path) -> Result<VerifyReport, CorpusError> {
    let manifest = parse_manifest(manifest_path)?;
    let mut entries = Vec::with_capacity(manifest.fixtures.len());
    let mut drifted = Vec::new();

    for entry in &manifest.fixtures {
        let jsx = manifest.corpus_root.join(&entry.jsx_path);
        let actual = hash_jsx_file(&jsx)?;
        if actual == entry.content_hash {
            entries.push(VerifyEntry {
                id: entry.id.clone(),
                status: FixtureStatus::Ok,
            });
        } else {
            drifted.push(DriftedFixture {
                id: entry.id.clone(),
                expected: entry.content_hash.clone(),
                actual: actual.clone(),
            });
            entries.push(VerifyEntry {
                id: entry.id.clone(),
                status: FixtureStatus::Drift {
                    expected: entry.content_hash.clone(),
                    actual,
                },
            });
        }
    }

    let total = entries.len();
    let clean = drifted.is_empty();
    Ok(VerifyReport {
        total,
        clean,
        entries,
        drifted,
    })
}
// CODEGEN-END

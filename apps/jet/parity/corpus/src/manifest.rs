// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-parity-corpus-src.md#schema
// CODEGEN-BEGIN
//! Manifest parser + schema validation for `fixtures.toml`.

use std::collections::HashSet;
use std::fmt;
use std::path::{Path, PathBuf};

use regex::Regex;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Canonical observation-channel enum. Wire form is kebab-case.
/// @spec .aw/tech-design/projects/jet/semantic/jet-parity-corpus-src.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ObservationChannel {
    Pixel,
    AxTree,
    FocusOrder,
    PointerHitMap,
    ImeTrace,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-parity-corpus-src.md#schema
impl ObservationChannel {
    pub fn as_kebab(&self) -> &'static str {
        match self {
            ObservationChannel::Pixel => "pixel",
            ObservationChannel::AxTree => "ax-tree",
            ObservationChannel::FocusOrder => "focus-order",
            ObservationChannel::PointerHitMap => "pointer-hit-map",
            ObservationChannel::ImeTrace => "ime-trace",
        }
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-parity-corpus-src.md#schema
impl fmt::Display for ObservationChannel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_kebab())
    }
}

/// One row of `fixtures.toml`.
///
/// @spec .aw/tech-design/projects/jet/specs/jet-parity-fixture-corpus.md#Dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixtureEntry {
    pub id: String,
    pub component: String,
    pub jsx_path: String,
    pub observation_channels: Vec<ObservationChannel>,
    pub mui_demo_source_url: String,
    pub content_hash: String,
}

/// Parsed `fixtures.toml`, carrying the corpus root and every entry.
/// @spec .aw/tech-design/projects/jet/semantic/jet-parity-corpus-src.md#schema
#[derive(Debug, Clone)]
pub struct FixtureManifest {
    /// Directory containing `fixtures.toml`. All `jsx_path` fields resolve
    /// relative to this root.
    pub corpus_root: PathBuf,
    pub fixtures: Vec<FixtureEntry>,
}

#[derive(Debug, Deserialize)]
struct RawManifest {
    #[serde(default)]
    fixtures: Vec<FixtureEntry>,
}

/// Error taxonomy for corpus parsing / hashing / verifying.
///
/// @spec .aw/tech-design/projects/jet/specs/jet-parity-fixture-corpus.md#Dependency
#[derive(Debug, Error)]
pub enum CorpusError {
    #[error("manifest not found: {0}")]
    ManifestNotFound(PathBuf),
    #[error("toml parse error: {0}")]
    TomlParse(#[from] toml::de::Error),
    #[error("io error reading {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("malformed fixture id: {0} (expected ^mui-[a-z0-9]+(?:-[a-z0-9]+)*-v[0-9]+$)")]
    MalformedId(String),
    #[error("unknown observation channel: {0}")]
    UnknownChannel(String),
    #[error("duplicate fixture id: {0}")]
    DuplicateId(String),
    #[error("missing jsx file for fixture {id}: {path}")]
    MissingJsx { id: String, path: PathBuf },
    #[error("schema violation: field={field} reason={reason}")]
    SchemaViolation { field: String, reason: String },
    #[error("fixture {id} not found in manifest")]
    UnknownFixture { id: String },
    #[error("content hash drift for fixture {id}: expected {expected} actual {actual}")]
    HashMismatch {
        id: String,
        expected: String,
        actual: String,
    },
}

fn fixture_id_regex() -> Regex {
    // Matches e.g. `mui-button-primary-v1`, `mui-textfield-outlined-v1`.
    Regex::new(r"^mui-[a-z0-9]+(?:-[a-z0-9]+)*-v[0-9]+$").expect("static regex compiles")
}

/// Parse + validate `fixtures.toml` at `manifest_path`.
///
/// @spec .aw/tech-design/projects/jet/specs/jet-parity-fixture-corpus.md#Logic
pub fn parse_manifest(manifest_path: &Path) -> Result<FixtureManifest, CorpusError> {
    if !manifest_path.exists() {
        return Err(CorpusError::ManifestNotFound(manifest_path.to_path_buf()));
    }
    let raw = std::fs::read_to_string(manifest_path).map_err(|source| CorpusError::Io {
        path: manifest_path.to_path_buf(),
        source,
    })?;
    let parsed: RawManifest = toml::from_str(&raw)?;

    let corpus_root = manifest_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));

    let id_re = fixture_id_regex();
    let mut seen: HashSet<String> = HashSet::new();

    for entry in &parsed.fixtures {
        if !id_re.is_match(&entry.id) {
            return Err(CorpusError::MalformedId(entry.id.clone()));
        }
        if !seen.insert(entry.id.clone()) {
            return Err(CorpusError::DuplicateId(entry.id.clone()));
        }
        if entry.content_hash.len() != 64
            || !entry.content_hash.chars().all(|c| c.is_ascii_hexdigit())
        {
            return Err(CorpusError::SchemaViolation {
                field: "content_hash".into(),
                reason: format!(
                    "expected 64 lowercase hex chars, got {:?}",
                    entry.content_hash
                ),
            });
        }
        let jsx = corpus_root.join(&entry.jsx_path);
        if !jsx.exists() {
            return Err(CorpusError::MissingJsx {
                id: entry.id.clone(),
                path: jsx,
            });
        }
        if entry.mui_demo_source_url.trim().is_empty() {
            return Err(CorpusError::SchemaViolation {
                field: "mui_demo_source_url".into(),
                reason: format!("empty for fixture {}", entry.id),
            });
        }
    }

    Ok(FixtureManifest {
        corpus_root,
        fixtures: parsed.fixtures,
    })
}

/// `parse_manifest` rejects unknown observation-channel values during
/// `toml::from_str` because `ObservationChannel` is `#[serde(rename_all =
/// "kebab-case")]`. This helper exists so the CLI can produce a stable
/// `CorpusError::UnknownChannel` for clearer messages when the raw TOML
/// uses a string surface form.
/// @spec .aw/tech-design/projects/jet/semantic/jet-parity-corpus-src.md#schema
pub fn from_kebab(raw: &str) -> Result<ObservationChannel, CorpusError> {
    match raw {
        "pixel" => Ok(ObservationChannel::Pixel),
        "ax-tree" => Ok(ObservationChannel::AxTree),
        "focus-order" => Ok(ObservationChannel::FocusOrder),
        "pointer-hit-map" => Ok(ObservationChannel::PointerHitMap),
        "ime-trace" => Ok(ObservationChannel::ImeTrace),
        other => Err(CorpusError::UnknownChannel(other.to_string())),
    }
}
// CODEGEN-END

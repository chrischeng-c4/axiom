// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-parity-gate-src.md#schema
// CODEGEN-BEGIN
//! `*.channel-result.json` schema (R2).

use std::path::Path;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use crate::manifest::GateError;

/// @spec .aw/tech-design/projects/jet/semantic/jet-parity-gate-src.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Status {
    Pass,
    Fail,
    Waived,
    Skipped,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-parity-gate-src.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DiffKind {
    PixelL2,
    AxTreeNodeCount,
    FocusOrderEditDistance,
    HitMapCellDiff,
    ImeEventSeqDiff,
}

/// One row in the gate's input set. Produced by the oracle per
/// (fixture, channel). See `docs/gating-manifest.md` § Channel-result
/// schema for the field-level contract.
/// @spec .aw/tech-design/projects/jet/semantic/jet-parity-gate-src.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelResult {
    pub schema_version: u32,
    pub fixture_id: String,
    pub channel: String,
    pub status: Status,
    pub diff_kind: DiffKind,
    pub diff_value: f64,
    pub captured_at: DateTime<Utc>,
    #[serde(default)]
    pub waived_by: Option<String>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-parity-gate-src.md#schema
impl ChannelResult {
    /// Walk `dir` recursively and parse every `*.channel-result.json`.
    /// Returns `Ok(vec![])` when `dir` does not exist (mirrors
    /// "skipped" semantics — the gate maps an empty set to exit 77).
    pub fn parse_dir(dir: impl AsRef<Path>) -> Result<Vec<ChannelResult>, GateError> {
        let dir_ref = dir.as_ref();
        if !dir_ref.exists() {
            return Ok(vec![]);
        }
        let mut out = Vec::new();
        for entry in WalkDir::new(dir_ref).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let name = match path.file_name().and_then(|n| n.to_str()) {
                Some(n) => n,
                None => continue,
            };
            if !name.ends_with(".channel-result.json") {
                continue;
            }
            let txt = std::fs::read_to_string(path).map_err(|e| GateError::Io {
                path: path.display().to_string(),
                source: e,
            })?;
            let row: ChannelResult = serde_json::from_str(&txt).map_err(|e| GateError::Json {
                path: path.display().to_string(),
                source: e,
            })?;
            out.push(row);
        }
        // Stable order so test assertions don't flake on FS ordering.
        out.sort_by(|a, b| {
            (a.fixture_id.as_str(), a.channel.as_str())
                .cmp(&(b.fixture_id.as_str(), b.channel.as_str()))
        });
        Ok(out)
    }
}
// CODEGEN-END

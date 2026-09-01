// HANDWRITE-BEGIN gap="missing-generator:logic:4643a21b" tracker="1873" reason="Persist collector.checkpoint.v1 by atomic fsynced replace and collector.rejection.v1 by bounded append diagnostics."
use std::path::Path;

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

pub const CHECKPOINT_SCHEMA: &str = "collector.checkpoint.v1";
pub const REJECTION_SCHEMA: &str = "collector.rejection.v1";
pub const MAX_REJECTION_PREVIEW_BYTES: usize = 1024;
pub const MAX_REJECTION_ERROR_BYTES: usize = 512;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct CollectorCheckpoint {
    pub schema: String,
    pub source_id: String,
    pub offset: u64,
    pub line: u64,
    pub accepted: u64,
    pub duplicates: u64,
    pub rejected: u64,
}

impl CollectorCheckpoint {
    pub fn new(source_id: impl Into<String>) -> Self {
        Self {
            schema: CHECKPOINT_SCHEMA.to_string(),
            source_id: source_id.into(),
            offset: 0,
            line: 0,
            accepted: 0,
            duplicates: 0,
            rejected: 0,
        }
    }

    pub fn load(path: &Path, source_id: &str) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::new(source_id));
        }
        let checkpoint: Self = service_collector::load_json_checkpoint(path)?
            .context("collector checkpoint disappeared while loading")?;
        if checkpoint.schema != CHECKPOINT_SCHEMA {
            bail!(
                "unsupported collector checkpoint schema {}; expected {}",
                checkpoint.schema,
                CHECKPOINT_SCHEMA
            );
        }
        if checkpoint.source_id != source_id {
            bail!(
                "collector checkpoint source mismatch: stored {}, configured {source_id}",
                checkpoint.source_id
            );
        }
        Ok(checkpoint)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        service_collector::save_json_checkpoint(path, self)
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct QuarantineEntry {
    pub schema: String,
    pub source_id: String,
    pub line: u64,
    pub offset: u64,
    pub code: String,
    pub message: String,
    pub preview: String,
}

impl QuarantineEntry {
    pub fn invalid_line(
        source_id: &str,
        line: u64,
        offset: u64,
        code: impl Into<String>,
        message: impl AsRef<str>,
        bytes: &[u8],
    ) -> Self {
        Self {
            schema: REJECTION_SCHEMA.to_string(),
            source_id: source_id.to_string(),
            line,
            offset,
            code: code.into(),
            message: truncate_utf8(message.as_ref(), MAX_REJECTION_ERROR_BYTES),
            preview: truncate_utf8(&String::from_utf8_lossy(bytes), MAX_REJECTION_PREVIEW_BYTES),
        }
    }
}

fn truncate_utf8(value: &str, max_bytes: usize) -> String {
    if value.len() <= max_bytes {
        return value.to_string();
    }
    let mut end = max_bytes;
    while !value.is_char_boundary(end) {
        end -= 1;
    }
    value[..end].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checkpoint_round_trip_is_source_bound_and_atomic() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("collector.json");
        let mut checkpoint = CollectorCheckpoint::new("file:/logs/lumen.jsonl");
        checkpoint.offset = 42;
        checkpoint.line = 3;
        checkpoint.accepted = 2;
        checkpoint.save(&path).unwrap();

        assert_eq!(
            CollectorCheckpoint::load(&path, "file:/logs/lumen.jsonl").unwrap(),
            checkpoint
        );
        assert!(CollectorCheckpoint::load(&path, "other-source").is_err());
    }

    #[test]
    fn quarantine_is_jsonl_and_bounds_untrusted_text() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("rejected.jsonl");
        let entry = QuarantineEntry::invalid_line(
            "fixture",
            7,
            99,
            "invalid_json",
            "e".repeat(MAX_REJECTION_ERROR_BYTES + 10),
            "x".repeat(MAX_REJECTION_PREVIEW_BYTES + 10).as_bytes(),
        );
        service_collector::append_jsonl(&path, &[entry]).unwrap();
        let line = std::fs::read_to_string(path).unwrap();
        let decoded: QuarantineEntry = serde_json::from_str(line.trim()).unwrap();
        assert_eq!(decoded.message.len(), MAX_REJECTION_ERROR_BYTES);
        assert_eq!(decoded.preview.len(), MAX_REJECTION_PREVIEW_BYTES);
    }
}
// HANDWRITE-END

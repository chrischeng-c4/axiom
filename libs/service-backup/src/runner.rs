// CODEGEN-BEGIN
//! One backup run: put the bytes, then apply retention.
//!
//! Three things are deliberately outside this primitive, and each is what makes
//! it testable:
//!
//! - **The clock.** `timestamp` is an argument, not `SystemTime::now()`. The
//!   sink derives the object key from it, so the caller -- not this function --
//!   decides which key a run writes.
//! - **Consistency.** The payload must already be a consistent snapshot. Nothing
//!   here quiesces a writer or reads an applied index; `raft-runtime` and the
//!   service's own admin snapshot endpoint own that.
//! - **The sink.** Retention runs after a successful `put` and only when
//!   `max_age_seconds` is `Some`.
//!
//! Two values in [`BackupRunResult`] are less
//! informative than they look. `pruned == 0` covers both "retention was
//! disabled" and "retention ran and nothing was old enough" -- the two are
//! indistinguishable from the result alone. And `unix_seconds` comes from
//! `duration_since(UNIX_EPOCH).unwrap_or_default()`, so a pre-epoch `timestamp`
//! is recorded as `0` rather than refused.
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{BackupSink, RetentionPolicy};

/// Object written by one backup run.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BackupObject {
    pub sink: String,
    pub key: String,
    pub bytes: usize,
    pub unix_seconds: u64,
}

/// Summary returned by a runner after upload + retention.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BackupRunResult {
    pub object: BackupObject,
    pub pruned: usize,
}

/// Write one already-consistent snapshot payload to a sink and apply retention.
pub fn run_backup_once(
    sink: &dyn BackupSink,
    timestamp: SystemTime,
    payload: &[u8],
    retention: &RetentionPolicy,
) -> Result<BackupRunResult> {
    let key = sink.put(timestamp, payload)?;
    let pruned = match retention.max_age_seconds {
        Some(max_age_seconds) => sink.prune(max_age_seconds)?,
        None => 0,
    };
    let unix_seconds = timestamp
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    Ok(BackupRunResult {
        object: BackupObject {
            sink: sink.identity(),
            key,
            bytes: payload.len(),
            unix_seconds,
        },
        pruned,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::LocalFsSink;

    #[test]
    fn runner_reports_written_object() {
        let dir =
            std::env::temp_dir().join(format!("service-backup-runner-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let sink = LocalFsSink::new(&dir, "svc").unwrap();
        let result = run_backup_once(
            &sink,
            SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(10),
            b"abc",
            &RetentionPolicy::default(),
        )
        .unwrap();
        assert_eq!(result.object.bytes, 3);
        assert_eq!(result.object.unix_seconds, 10);
        assert!(dir.join(result.object.key).exists());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
// CODEGEN-END

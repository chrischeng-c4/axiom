---
id: libs-service-backup-src-runner-rs
summary: Lossless rust-source-unit coverage for `libs/service-backup/src/runner.rs`.
capability_refs:
  - id: shared-service-backup-contract
    role: primary
    claim: shared-service-backup-contract
    coverage: full
    rationale: "The source, tests, and manifest implement the Service Backup library contract."
fill_sections: [overview, source, changes]
---

# Standardized libs/service-backup/src/runner.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/service-backup/src/runner.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `BackupObject` | libs/service-backup/src/runner.rs | struct | pub | 12 | pub struct BackupObject { |
| `BackupRunResult` | libs/service-backup/src/runner.rs | struct | pub | 22 | pub struct BackupRunResult { |
| `run_backup_once` | libs/service-backup/src/runner.rs | function | pub | 28 | pub fn run_backup_once( |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
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
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/service-backup/src/runner.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/service-backup/src/runner.rs` captured during libs codegen standardization.
```

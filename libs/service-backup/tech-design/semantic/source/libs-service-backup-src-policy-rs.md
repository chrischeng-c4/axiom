---
id: libs-service-backup-src-policy-rs
summary: Lossless rust-source-unit coverage for `libs/service-backup/src/policy.rs`.
capability_refs:
  - id: shared-service-backup-contract
    role: primary
    claim: shared-service-backup-contract
    coverage: full
    rationale: "The source, tests, and manifest implement the Service Backup library contract."
fill_sections: [overview, source, changes]
---

# Standardized libs/service-backup/src/policy.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/service-backup/src/policy.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `BackupPolicy` | libs/service-backup/src/policy.rs | struct | pub | 9 | pub struct BackupPolicy { |
| `RetentionPolicy` | libs/service-backup/src/policy.rs | struct | pub | 21 | pub struct RetentionPolicy { |
| `max_age_seconds` | libs/service-backup/src/policy.rs | function | pub | 28 | pub fn max_age_seconds(max_age_seconds: u64) -> Self { |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::BackupDestination;

/// Operator/runner-facing backup policy.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BackupPolicy {
    /// Cron expression for the runner. The operator owns translating this into
    /// a Kubernetes CronJob schedule.
    pub schedule: String,
    pub destination: BackupDestination,
    #[serde(default)]
    pub retention: RetentionPolicy,
}

/// Retention applied after a successful put.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RetentionPolicy {
    /// Drop objects older than this many seconds. `None` disables age pruning.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_age_seconds: Option<u64>,
}

impl RetentionPolicy {
    pub fn max_age_seconds(max_age_seconds: u64) -> Self {
        Self {
            max_age_seconds: Some(max_age_seconds),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn policy_serializes_camel_case() {
        let p = BackupPolicy {
            schedule: "0 * * * *".into(),
            destination: BackupDestination::from_uri("s3://b/p").unwrap(),
            retention: RetentionPolicy::max_age_seconds(3600),
        };
        let json = serde_json::to_value(&p).unwrap();
        assert_eq!(json["retention"]["maxAgeSeconds"], 3600);
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/service-backup/src/policy.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/service-backup/src/policy.rs` captured during libs codegen standardization.
```

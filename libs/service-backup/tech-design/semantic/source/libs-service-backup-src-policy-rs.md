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
| `ScheduledBackupPolicy` | libs/service-backup/src/policy.rs | struct | pub | — | pub struct ScheduledBackupPolicy { |
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

/// Kubernetes structural-schema-safe scheduled backup projection.
///
/// The runtime [`BackupPolicy`] uses a tagged [`BackupDestination`] enum whose
/// variant schemas cannot be embedded directly in a CRD. This flat shape keeps
/// the public `schedule`, `destination`, and `retentionSecs` fields shared by
/// every service operator and validates them through one conversion path.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ScheduledBackupPolicy {
    /// Cron expression rendered into `CronJob.spec.schedule`.
    pub schedule: String,
    /// Destination URI accepted by [`BackupDestination::from_uri`].
    pub destination: String,
    /// Drop objects older than this many seconds after a successful put.
    /// `None` keeps every object.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retention_secs: Option<u64>,
}

impl ScheduledBackupPolicy {
    /// Validate the flat CRD projection and create the runtime policy.
    pub fn to_runtime_policy(&self) -> anyhow::Result<BackupPolicy> {
        if self.schedule.trim().is_empty() {
            anyhow::bail!("backup schedule must not be empty");
        }
        Ok(BackupPolicy {
            schedule: self.schedule.clone(),
            destination: BackupDestination::from_uri(self.destination.trim())?,
            retention: RetentionPolicy {
                max_age_seconds: self.retention_secs,
            },
        })
    }
}

impl TryFrom<&ScheduledBackupPolicy> for BackupPolicy {
    type Error = anyhow::Error;

    fn try_from(policy: &ScheduledBackupPolicy) -> Result<Self, Self::Error> {
        policy.to_runtime_policy()
    }
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

    #[test]
    fn scheduled_policy_is_flat_and_structural_schema_safe() {
        let policy = ScheduledBackupPolicy {
            schedule: "0 * * * *".into(),
            destination: "s3://bucket/prefix".into(),
            retention_secs: Some(3600),
        };
        let json = serde_json::to_value(&policy).unwrap();
        assert_eq!(
            json,
            serde_json::json!({
                "schedule": "0 * * * *",
                "destination": "s3://bucket/prefix",
                "retentionSecs": 3600,
            })
        );

        let schema = serde_json::to_value(schemars::schema_for!(ScheduledBackupPolicy)).unwrap();
        let properties = schema["properties"].as_object().unwrap();
        assert_eq!(properties["destination"]["type"], "string");
        assert!(properties.contains_key("schedule"));
        assert!(properties.contains_key("retentionSecs"));
        assert!(schema.get("oneOf").is_none());
    }

    #[test]
    fn scheduled_policy_uses_one_validated_runtime_conversion() {
        let policy = ScheduledBackupPolicy {
            schedule: "0 * * * *".into(),
            destination: "s3://bucket/prefix".into(),
            retention_secs: Some(3600),
        };
        let runtime = policy.to_runtime_policy().unwrap();
        assert_eq!(runtime.schedule, policy.schedule);
        assert_eq!(runtime.destination.identity(), "s3://bucket/prefix");
        assert_eq!(runtime.retention.max_age_seconds, Some(3600));

        let mut invalid = policy.clone();
        invalid.schedule = "  ".into();
        assert!(invalid.to_runtime_policy().is_err());
        invalid.schedule = "0 * * * *".into();
        invalid.destination = "ftp://bucket/prefix".into();
        assert!(invalid.to_runtime_policy().is_err());
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

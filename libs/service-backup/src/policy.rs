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

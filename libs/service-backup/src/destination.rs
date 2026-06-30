use anyhow::{bail, ensure, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Backup destination declared by a service CR or runner config.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum BackupDestination {
    /// Local filesystem path, primarily for dev/tests and PVC-backed local runs.
    Local {
        path: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        prefix: Option<String>,
    },
    /// Amazon S3-compatible object store. Upload implementation is a crate
    /// feature; the schema is stable regardless of whether that feature is
    /// linked into the runner.
    S3 {
        bucket: String,
        #[serde(default)]
        prefix: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        region: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        endpoint: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        credentials_secret: Option<String>,
    },
    /// Google Cloud Storage. Prefer workload identity; use
    /// `credentialsSecret` only for explicit key-based deployments.
    Gcs {
        bucket: String,
        #[serde(default)]
        prefix: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        credentials_secret: Option<String>,
    },
}

impl BackupDestination {
    /// Parse the common URI spellings used by operators and CLIs:
    /// `file:///path`, `s3://bucket/prefix`, and `gs://bucket/prefix`.
    pub fn from_uri(raw: &str) -> Result<Self> {
        let raw = raw.trim();
        ensure!(!raw.is_empty(), "backup destination URI is empty");
        if let Some(path) = raw.strip_prefix("file://") {
            ensure!(!path.is_empty(), "file backup URI has no path");
            return Ok(Self::Local {
                path: path.to_string(),
                prefix: None,
            });
        }
        if let Some(rest) = raw.strip_prefix("s3://") {
            let (bucket, prefix) = split_bucket_prefix(rest, "s3")?;
            return Ok(Self::S3 {
                bucket,
                prefix,
                region: None,
                endpoint: None,
                credentials_secret: None,
            });
        }
        if let Some(rest) = raw.strip_prefix("gs://") {
            let (bucket, prefix) = split_bucket_prefix(rest, "gs")?;
            return Ok(Self::Gcs {
                bucket,
                prefix,
                credentials_secret: None,
            });
        }
        bail!("unsupported backup destination URI `{raw}`; use file://, s3://, or gs://")
    }

    pub fn identity(&self) -> String {
        match self {
            Self::Local { path, .. } => format!("local:{path}"),
            Self::S3 { bucket, prefix, .. } if prefix.is_empty() => format!("s3://{bucket}"),
            Self::S3 { bucket, prefix, .. } => format!("s3://{bucket}/{prefix}"),
            Self::Gcs { bucket, prefix, .. } if prefix.is_empty() => format!("gs://{bucket}"),
            Self::Gcs { bucket, prefix, .. } => format!("gs://{bucket}/{prefix}"),
        }
    }

    pub fn default_prefix(&self) -> String {
        match self {
            Self::Local { prefix, .. } => prefix.clone().unwrap_or_else(|| "backup".into()),
            Self::S3 { prefix, .. } | Self::Gcs { prefix, .. } if prefix.is_empty() => {
                "backup".into()
            }
            Self::S3 { prefix, .. } | Self::Gcs { prefix, .. } => prefix.clone(),
        }
    }
}

fn split_bucket_prefix(rest: &str, scheme: &str) -> Result<(String, String)> {
    let rest = rest.trim_end_matches('/');
    let Some((bucket, prefix)) = rest.split_once('/') else {
        ensure!(!rest.is_empty(), "{scheme} backup URI has no bucket");
        return Ok((rest.to_string(), String::new()));
    };
    ensure!(!bucket.is_empty(), "{scheme} backup URI has no bucket");
    Ok((bucket.to_string(), prefix.trim_matches('/').to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_object_store_uris() {
        assert_eq!(
            BackupDestination::from_uri("s3://bucket/a/b")
                .unwrap()
                .identity(),
            "s3://bucket/a/b"
        );
        assert_eq!(
            BackupDestination::from_uri("gs://bucket/prefix")
                .unwrap()
                .identity(),
            "gs://bucket/prefix"
        );
        assert_eq!(
            BackupDestination::from_uri("file:///tmp/backups")
                .unwrap()
                .identity(),
            "local:/tmp/backups"
        );
    }

    #[test]
    fn rejects_missing_bucket() {
        assert!(BackupDestination::from_uri("s3:///prefix").is_err());
        assert!(BackupDestination::from_uri("gs://").is_err());
    }
}

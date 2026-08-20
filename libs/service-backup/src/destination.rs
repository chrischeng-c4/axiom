// SPEC-MANAGED: libs/service-backup/tech-design/semantic/source/libs-service-backup-src-destination-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! Where a backup goes, in the two spellings operators actually use.
//!
//! [`BackupDestination`] is an internally tagged enum
//! (`#[serde(tag = "type")]`), which is exactly why it cannot be embedded in a
//! Kubernetes structural schema and why
//! [`ScheduledBackupPolicy`](crate::ScheduledBackupPolicy) exists as a flat
//! projection beside it.
//!
//! The URI form is strictly less expressive than the CR form. `from_uri` fills
//! in `bucket` and `prefix` and leaves `region`, `endpoint` and
//! `credentials_secret` as `None` every time -- those can only arrive by
//! deserializing a CR, never from a URI. `default_prefix` then collapses "no
//! prefix" (local `None`, object-store `""`) to the literal `"backup"`, so an
//! empty prefix never reaches a sink as empty.
//!
//! [`SUPPORTED_SCHEMES`] is the canonical inventory,
//! ordered to match `from_uri`'s parse order, and two unit tests hold it to
//! that: one asserts the rejection message names every scheme in the table, the
//! other asserts every scheme in the table parses a well-formed URI. The CLI
//! `llm` topics render the table at call time instead of copying it into a
//! string, so a help body cannot drift from what this file accepts (#2494).
//!
//! `sink_available` is the one column that is **not** about parsing. It is
//! `cfg!(feature = ...)` for this build, and a `false` there does not stop
//! `from_uri` from succeeding -- `s3://` parses in every build, and a build
//! without the `s3` feature only fails later, inside
//! [`BackupSink::put`](crate::BackupSink::put), through
//! [`UnsupportedCloudSink`](crate::UnsupportedCloudSink). "Does this binary
//! support S3" is therefore a question about the sink, never about the URI.
use anyhow::{bail, ensure, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Backup destination declared by a service CR or runner config.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "lowercase")]
/// @spec libs/service-backup/tech-design/semantic/source/libs-service-backup-src-destination-rs.md#source
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
    /// Google Cloud Storage. The adapter uses workload identity by default and
    /// Vat's `STORAGE_EMULATOR_HOST` for integration tests.
    Gcs {
        bucket: String,
        #[serde(default)]
        prefix: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        credentials_secret: Option<String>,
    },
}

/// One backup destination scheme [`BackupDestination::from_uri`] accepts,
/// paired with a human-readable description and whether a live upload sink
/// is linked into this build (see `sink_from_destination` in `sink.rs`).
///
/// This is the canonical scheme inventory: CLI `llm` topics render it at
/// call time (`cli_std::llm::TopicSection::Generated`) instead of
/// hand-copying the scheme list into a `&'static str`, so a topic body can
/// never drift from what `from_uri`/`sink_from_destination` actually accept
/// (#2494).
#[derive(Clone, Copy, Debug)]
pub struct SchemeInfo {
    pub scheme: &'static str,
    pub description: &'static str,
    /// `false` means the scheme still parses via `from_uri`, but
    /// `BackupSink::put`/`prune` fail loud through `UnsupportedCloudSink`
    /// because the adapter crate feature isn't linked into this build.
    pub sink_available: bool,
}

/// Canonical scheme table backing [`BackupDestination::from_uri`], ordered
/// to match its parse order. `sink_available` reflects the *this build's*
/// linked feature set via `cfg!`, so it can't go stale independent of the
/// actual `Cargo.toml` feature wiring.
pub const SUPPORTED_SCHEMES: &[SchemeInfo] = &[
    SchemeInfo {
        scheme: "file://",
        description: "local filesystem path — dev/tests and PVC-backed local runs",
        sink_available: true,
    },
    SchemeInfo {
        scheme: "s3://",
        description: "Amazon S3-compatible object store",
        sink_available: cfg!(feature = "s3"),
    },
    SchemeInfo {
        scheme: "gs://",
        description: "Google Cloud Storage — workload identity in production, `STORAGE_EMULATOR_HOST` locally",
        sink_available: true,
    },
];

/// @spec libs/service-backup/tech-design/semantic/source/libs-service-backup-src-destination-rs.md#source
impl BackupDestination {
    /// Parse the common URI spellings used by operators and CLIs.
    /// `gs://bucket/prefix` selects the always-linked GCS adapter.
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
        bail!(
            "unsupported backup destination URI `{raw}`; use {}",
            SUPPORTED_SCHEMES
                .iter()
                .map(|s| s.scheme)
                .collect::<Vec<_>>()
                .join(", ")
        )
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

    #[test]
    fn supported_schemes_match_from_uri_error_message() {
        let err = BackupDestination::from_uri("ftp://nope")
            .unwrap_err()
            .to_string();
        for info in SUPPORTED_SCHEMES {
            assert!(
                err.contains(info.scheme),
                "error message {err:?} missing scheme {}",
                info.scheme
            );
        }
    }

    #[test]
    fn supported_schemes_each_parse_successfully() {
        for info in SUPPORTED_SCHEMES {
            let uri = match info.scheme {
                "file://" => "file:///tmp/backups".to_string(),
                other => format!("{other}bucket/prefix"),
            };
            assert!(
                BackupDestination::from_uri(&uri).is_ok(),
                "scheme {} failed to parse a well-formed URI",
                info.scheme
            );
        }
    }

    #[test]
    fn local_and_gcs_sinks_always_available() {
        let by_scheme = |scheme: &str| SUPPORTED_SCHEMES.iter().find(|s| s.scheme == scheme);
        assert!(by_scheme("file://").unwrap().sink_available);
        assert!(by_scheme("gs://").unwrap().sink_available);
    }
}
// CODEGEN-END

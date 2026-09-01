// HANDWRITE-BEGIN gap="missing-generator:logic:ca755a11" tracker="1873" reason="Own CollectorConfig, SourceSpec, CollectorSummary, validation defaults, module exports, and run_collector."
use std::path::PathBuf;
use std::time::Duration;

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

mod checkpoint;
mod client;
mod cri;
mod model;
mod runtime;
mod source;

pub use checkpoint::{CollectorCheckpoint, QuarantineEntry};
pub use model::decode_service_log;

pub const DEFAULT_BATCH_SIZE: usize = 100;
pub const MAX_BATCH_SIZE: usize = 1000;
pub const DEFAULT_MAX_LINE_BYTES: usize = 512 * 1024;
pub const MAX_LINE_BYTES: usize = 1024 * 1024;
pub const DEFAULT_MAX_RETRIES: usize = 3;

// <HANDWRITE gap="missing-generator:logic" tracker="1675" reason="Add typed CRI source configuration and metadata over the shared collector runtime.">
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CriMetadata {
    pub gcp_project: String,
    pub cluster: Option<String>,
    pub location: Option<String>,
    pub node: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CriSourceConfig {
    pub root: PathBuf,
    pub metadata: CriMetadata,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SourceSpec {
    File(PathBuf),
    Stdin,
    Cri(CriSourceConfig),
}
// </HANDWRITE>

#[derive(Clone, Debug)]
pub struct CollectorConfig {
    pub source: SourceSpec,
    pub source_id: String,
    pub endpoint: String,
    pub token: Option<String>,
    pub token_file: Option<PathBuf>,
    pub token_audience: String,
    pub project: String,
    pub environment: String,
    pub checkpoint_path: PathBuf,
    pub quarantine_path: PathBuf,
    pub batch_size: usize,
    pub max_line_bytes: usize,
    pub max_retries: usize,
    pub request_timeout: Duration,
    pub initial_backoff: Duration,
    pub follow: bool,
    pub follow_poll_interval: Duration,
}

impl CollectorConfig {
    pub fn validate(&self) -> Result<()> {
        for (name, value) in [
            ("source_id", self.source_id.as_str()),
            ("endpoint", self.endpoint.as_str()),
            ("project", self.project.as_str()),
            ("environment", self.environment.as_str()),
            ("token_audience", self.token_audience.as_str()),
        ] {
            if value.trim().is_empty() {
                bail!("collector {name} must not be empty");
            }
            if value.len() > 512 {
                bail!("collector {name} must not exceed 512 bytes");
            }
        }
        if self.token.is_some() && self.token_file.is_some() {
            bail!("collector token and token_file are mutually exclusive");
        }
        if self.batch_size == 0 || self.batch_size > MAX_BATCH_SIZE {
            bail!("collector batch_size must be between 1 and {MAX_BATCH_SIZE}");
        }
        if self.max_line_bytes == 0 || self.max_line_bytes > MAX_LINE_BYTES {
            bail!("collector max_line_bytes must be between 1 and {MAX_LINE_BYTES}");
        }
        if self.max_retries > 20 {
            bail!("collector max_retries must not exceed 20");
        }
        if self.request_timeout.is_zero()
            || self.initial_backoff.is_zero()
            || self.follow_poll_interval.is_zero()
        {
            bail!("collector timeout and polling durations must be nonzero");
        }
        if self.follow && matches!(self.source, SourceSpec::Stdin) {
            bail!("collector follow mode requires a regular file or CRI source");
        }
        if self.checkpoint_path == self.quarantine_path {
            bail!("collector checkpoint and quarantine paths must differ");
        }
        if let SourceSpec::Cri(cri) = &self.source {
            validate_metadata("gcp_project", &cri.metadata.gcp_project)?;
            for (name, value) in [
                ("cluster", cri.metadata.cluster.as_deref()),
                ("location", cri.metadata.location.as_deref()),
                ("node", cri.metadata.node.as_deref()),
            ] {
                if let Some(value) = value {
                    validate_metadata(name, value)?;
                }
            }
        }
        Ok(())
    }
}

fn validate_metadata(name: &str, value: &str) -> Result<()> {
    if value.trim().is_empty() || value.len() > 512 || value.chars().any(char::is_control) {
        bail!("collector CRI {name} must be nonempty, bounded, and control-character-free");
    }
    Ok(())
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct CollectorSummary {
    pub source_id: String,
    pub start_offset: u64,
    pub final_offset: u64,
    pub lines: u64,
    pub accepted: u64,
    pub duplicates: u64,
    pub rejected: u64,
    pub lost_bytes: u64,
    pub lost_sources: u64,
}

pub async fn run_collector(config: CollectorConfig) -> Result<CollectorSummary> {
    config.validate()?;
    runtime::run(config).await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(source: SourceSpec) -> CollectorConfig {
        CollectorConfig {
            source,
            source_id: "fixture".to_string(),
            endpoint: "http://127.0.0.1:7380".to_string(),
            token: None,
            token_file: None,
            token_audience: "sift.axiom.dev".to_string(),
            project: "local".to_string(),
            environment: "test".to_string(),
            checkpoint_path: "checkpoint.json".into(),
            quarantine_path: "rejected.jsonl".into(),
            batch_size: DEFAULT_BATCH_SIZE,
            max_line_bytes: DEFAULT_MAX_LINE_BYTES,
            max_retries: DEFAULT_MAX_RETRIES,
            request_timeout: Duration::from_secs(5),
            initial_backoff: Duration::from_millis(50),
            follow: false,
            follow_poll_interval: Duration::from_millis(100),
        }
    }

    #[test]
    fn config_bounds_batch_line_retry_and_follow() {
        assert!(config(SourceSpec::Stdin).validate().is_ok());
        let mut invalid = config(SourceSpec::Stdin);
        invalid.follow = true;
        assert!(invalid.validate().is_err());

        let mut invalid = config(SourceSpec::File("logs.jsonl".into()));
        invalid.batch_size = 0;
        assert!(invalid.validate().is_err());

        let mut invalid = config(SourceSpec::File("logs.jsonl".into()));
        invalid.max_line_bytes = MAX_LINE_BYTES + 1;
        assert!(invalid.validate().is_err());
    }
}
// HANDWRITE-END

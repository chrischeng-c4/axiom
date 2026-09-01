// HANDWRITE-BEGIN gap="missing-generator:logic:d9e63ee1" tracker="1873" reason="Run the sole bounded decode, quarantine, delivery, ack, and checkpoint loop over source-neutral records."
use std::time::Duration;

use anyhow::Result;

use super::checkpoint::QuarantineEntry;
use super::client::CollectorClient;
use super::model::decode_service_log_enriched;
use super::source::{open_source, RawRecord};
use super::{CollectorConfig, CollectorSummary};

struct SiftRecordDecoder<'a> {
    project: &'a str,
    environment: &'a str,
}

impl service_collector::RecordDecoder<RawRecord> for SiftRecordDecoder<'_> {
    type Item = crate::OperationalEventV2;
    type Rejection = QuarantineEntry;

    fn decode(&self, record: RawRecord) -> Result<Self::Item, Self::Rejection> {
        decode_service_log_enriched(
            &record.bytes,
            &record.source_id,
            record.offset,
            self.project,
            self.environment,
            &record.enrichment,
        )
        .map_err(|error| {
            QuarantineEntry::invalid_line(
                &record.source_id,
                record.line,
                record.offset,
                "invalid_service_log",
                error.to_string(),
                &record.bytes,
            )
        })
    }
}

// <HANDWRITE gap="missing-generator:logic" tracker="1675" reason="Drive file, stdin, and CRI through one shared collector core.">
pub async fn run(config: CollectorConfig) -> Result<CollectorSummary> {
    let mut source = open_source(&config)?;
    let mut client = CollectorClient::new(
        &config.endpoint,
        &config.project,
        config.token.clone(),
        config.request_timeout,
    )?;
    if let Some(path) = &config.token_file {
        client = client.with_projected_token_file(path.clone(), config.token_audience.clone());
    }
    let mut quarantine =
        service_collector::JsonlQuarantine::<QuarantineEntry>::new(&config.quarantine_path);
    let report = service_collector::run_collector(
        &mut *source,
        &SiftRecordDecoder {
            project: &config.project,
            environment: &config.environment,
        },
        &client,
        &mut quarantine,
        service_collector::RuntimeConfig {
            batch_size: config.batch_size,
            max_record_bytes: config.max_line_bytes,
            retry: service_collector::RetryPolicy::new(
                config.max_retries,
                config.initial_backoff,
                Duration::from_secs(5),
            )?,
            follow: config.follow,
            follow_poll_interval: config.follow_poll_interval,
        },
    )
    .await?;

    Ok(CollectorSummary {
        source_id: config.source_id.clone(),
        start_offset: report.progress.start_offset,
        final_offset: report.progress.final_offset,
        lines: report.lines,
        accepted: report.accepted,
        duplicates: report.duplicates,
        rejected: report.rejected,
        lost_bytes: report.progress.lost_bytes,
        lost_sources: report.progress.lost_sources,
    })
}
// </HANDWRITE>

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use crate::collector::{
        CollectorCheckpoint, SourceSpec, DEFAULT_BATCH_SIZE, DEFAULT_MAX_LINE_BYTES,
        DEFAULT_MAX_RETRIES,
    };

    #[tokio::test]
    async fn truncated_file_is_refused_before_delivery() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("service.jsonl");
        let checkpoint_path = temp.path().join("checkpoint.json");
        std::fs::write(&source, b"{}\n").unwrap();
        let mut checkpoint = CollectorCheckpoint::new("fixture");
        checkpoint.offset = 100;
        checkpoint.save(&checkpoint_path).unwrap();
        let config = CollectorConfig {
            source: SourceSpec::File(source),
            source_id: "fixture".to_string(),
            endpoint: "http://127.0.0.1:7380".to_string(),
            token: None,
            token_file: None,
            token_audience: "sift.axiom.dev".to_string(),
            project: "local".to_string(),
            environment: "test".to_string(),
            checkpoint_path,
            quarantine_path: temp.path().join("rejected.jsonl"),
            batch_size: DEFAULT_BATCH_SIZE,
            max_line_bytes: DEFAULT_MAX_LINE_BYTES,
            max_retries: DEFAULT_MAX_RETRIES,
            request_timeout: Duration::from_secs(1),
            initial_backoff: Duration::from_millis(1),
            follow: false,
            follow_poll_interval: Duration::from_millis(1),
        };
        let error = run(config).await.unwrap_err().to_string();
        assert!(error.contains("truncated or rotated"));
    }
}
// HANDWRITE-END

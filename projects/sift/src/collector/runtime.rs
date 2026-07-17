// HANDWRITE-BEGIN gap="missing-generator:logic:d9e63ee1" tracker="1873" reason="Run the sole bounded decode, quarantine, delivery, ack, and checkpoint loop over source-neutral records."
use anyhow::Result;

use super::checkpoint::{append_quarantine, QuarantineEntry};
use super::client::CollectorClient;
use super::model::decode_service_log_enriched;
use super::source::{open_source, CommitStats, ReadOutcome, SourceCursor};
use super::{CollectorConfig, CollectorSummary};

// <HANDWRITE gap="missing-generator:logic" tracker="1675" reason="Drive file, stdin, and CRI through one shared collector core.">
pub async fn run(config: CollectorConfig) -> Result<CollectorSummary> {
    let mut source = open_source(&config)?;
    let client = CollectorClient::new(
        &config.endpoint,
        &config.project,
        config.token.clone(),
        config.max_retries,
        config.request_timeout,
        config.initial_backoff,
    )?;
    let mut summary = CollectorSummary {
        source_id: config.source_id.clone(),
        start_offset: source.start_offset(),
        final_offset: source.final_offset(),
        lost_bytes: source.lost_bytes(),
        lost_sources: source.lost_sources(),
        ..CollectorSummary::default()
    };

    loop {
        let mut events = Vec::with_capacity(config.batch_size);
        let mut rejections: Vec<QuarantineEntry> = Vec::new();
        let mut cursors: Vec<SourceCursor> = Vec::with_capacity(config.batch_size);
        let mut reached_end = false;

        for _ in 0..config.batch_size {
            match source.next_record(config.max_line_bytes)? {
                ReadOutcome::Record(record) => {
                    cursors.push(record.cursor.clone());
                    match decode_service_log_enriched(
                        &record.bytes,
                        &record.source_id,
                        record.offset,
                        &config.project,
                        &config.environment,
                        &record.enrichment,
                    ) {
                        Ok(event) => events.push(event),
                        Err(error) => rejections.push(QuarantineEntry::invalid_line(
                            &record.source_id,
                            record.line,
                            record.offset,
                            "invalid_service_log",
                            error.to_string(),
                            &record.bytes,
                        )),
                    }
                }
                ReadOutcome::Rejection(rejection) => {
                    cursors.push(rejection.cursor);
                    rejections.push(rejection.entry);
                }
                ReadOutcome::Pending | ReadOutcome::Exhausted => {
                    reached_end = true;
                    break;
                }
            }
        }

        if cursors.is_empty() {
            if config.follow {
                tokio::time::sleep(config.follow_poll_interval).await;
                source.refresh()?;
                continue;
            }
            break;
        }

        let delivered = client.send(&events).await?;
        append_quarantine(&config.quarantine_path, &rejections)?;
        source.commit(
            &cursors,
            CommitStats {
                accepted: delivered.accepted,
                duplicates: delivered.duplicates,
                rejected: rejections.len() as u64,
            },
        )?;

        summary.lines += cursors.len() as u64;
        summary.accepted += delivered.accepted;
        summary.duplicates += delivered.duplicates;
        summary.rejected += rejections.len() as u64;
        summary.final_offset = source.final_offset();
        summary.lost_bytes = source.lost_bytes();
        summary.lost_sources = source.lost_sources();

        if reached_end {
            if config.follow {
                tokio::time::sleep(config.follow_poll_interval).await;
                source.refresh()?;
            } else {
                break;
            }
        }
    }

    summary.final_offset = source.final_offset();
    summary.lost_bytes = source.lost_bytes();
    summary.lost_sources = source.lost_sources();
    Ok(summary)
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

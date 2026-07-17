// HANDWRITE-BEGIN gap="missing-generator:logic:d9e63ee1" tracker="1873" reason="Open/seek/discard file or stdin sources and run the bounded window, quarantine, delivery, checkpoint, one-shot, and follow loop."
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};

use anyhow::{bail, Context, Result};

use super::checkpoint::{append_quarantine, CollectorCheckpoint, QuarantineEntry};
use super::client::CollectorClient;
use super::model::decode_service_log;
use super::{CollectorConfig, CollectorSummary, SourceSpec};

// <HANDWRITE gap="missing-generator:logic" tracker="pending-tracker" reason="logic section in runtime.rs is hand-written pending codegen support">
pub async fn run(config: CollectorConfig) -> Result<CollectorSummary> {
    let checkpoint = CollectorCheckpoint::load(&config.checkpoint_path, &config.source_id)?;
    let client = CollectorClient::new(
        &config.endpoint,
        &config.project,
        config.token.clone(),
        config.max_retries,
        config.request_timeout,
        config.initial_backoff,
    )?;

    match &config.source {
        SourceSpec::File(path) => {
            let metadata = std::fs::metadata(path)
                .with_context(|| format!("inspect collector source {}", path.display()))?;
            if !metadata.is_file() {
                bail!("collector file source must be a regular file");
            }
            if metadata.len() < checkpoint.offset {
                bail!(
                    "collector source was truncated or rotated: size {} is below checkpoint {}; use a new source_id or the CRI rotation adapter tracked by #1675",
                    metadata.len(),
                    checkpoint.offset
                );
            }
            let mut file = File::open(path)
                .with_context(|| format!("open collector source {}", path.display()))?;
            file.seek(SeekFrom::Start(checkpoint.offset))
                .with_context(|| format!("seek collector source {}", path.display()))?;
            let mut reader = BufReader::new(file);
            collect_reader(&mut reader, &config, checkpoint, &client).await
        }
        SourceSpec::Stdin => {
            let stdin = std::io::stdin();
            let mut reader = BufReader::new(stdin.lock());
            discard_acknowledged(&mut reader, checkpoint.offset)?;
            collect_reader(&mut reader, &config, checkpoint, &client).await
        }
    }
}
// </HANDWRITE>

async fn collect_reader<R: BufRead>(
    reader: &mut R,
    config: &CollectorConfig,
    mut checkpoint: CollectorCheckpoint,
    client: &CollectorClient,
) -> Result<CollectorSummary> {
    let mut summary = CollectorSummary {
        source_id: config.source_id.clone(),
        start_offset: checkpoint.offset,
        final_offset: checkpoint.offset,
        ..CollectorSummary::default()
    };

    loop {
        let mut events = Vec::with_capacity(config.batch_size);
        let mut rejections = Vec::new();
        let mut window_offset = checkpoint.offset;
        let mut window_line = checkpoint.line;
        let mut reached_eof = false;

        for _ in 0..config.batch_size {
            let start_offset = window_offset;
            let read = read_bounded_line(reader, config.max_line_bytes)?;
            if read.bytes_read == 0 {
                reached_eof = true;
                break;
            }
            window_offset = window_offset
                .checked_add(read.bytes_read)
                .context("collector byte offset overflow")?;
            window_line = window_line
                .checked_add(1)
                .context("collector line counter overflow")?;
            if read.oversized {
                rejections.push(QuarantineEntry::invalid_line(
                    &config.source_id,
                    window_line,
                    start_offset,
                    "line_too_large",
                    format!(
                        "structured stdout line exceeds {} bytes",
                        config.max_line_bytes
                    ),
                    &read.preview,
                ));
                continue;
            }
            match decode_service_log(
                &read.preview,
                &config.source_id,
                start_offset,
                &config.project,
                &config.environment,
            ) {
                Ok(event) => events.push(event),
                Err(error) => rejections.push(QuarantineEntry::invalid_line(
                    &config.source_id,
                    window_line,
                    start_offset,
                    "invalid_service_log",
                    error.to_string(),
                    &read.preview,
                )),
            }
        }

        if window_line == checkpoint.line {
            if config.follow {
                tokio::time::sleep(config.follow_poll_interval).await;
                continue;
            }
            break;
        }

        let processed_lines = window_line - checkpoint.line;
        let delivered = client.send(&events).await?;
        append_quarantine(&config.quarantine_path, &rejections)?;

        checkpoint.offset = window_offset;
        checkpoint.line = window_line;
        checkpoint.accepted = checkpoint
            .accepted
            .checked_add(delivered.accepted)
            .context("collector accepted counter overflow")?;
        checkpoint.duplicates = checkpoint
            .duplicates
            .checked_add(delivered.duplicates)
            .context("collector duplicate counter overflow")?;
        checkpoint.rejected = checkpoint
            .rejected
            .checked_add(rejections.len() as u64)
            .context("collector rejected counter overflow")?;
        checkpoint.save(&config.checkpoint_path)?;

        summary.lines += processed_lines;
        summary.accepted += delivered.accepted;
        summary.duplicates += delivered.duplicates;
        summary.rejected += rejections.len() as u64;
        summary.final_offset = checkpoint.offset;

        if reached_eof {
            if config.follow {
                tokio::time::sleep(config.follow_poll_interval).await;
            } else {
                break;
            }
        }
    }

    Ok(summary)
}

struct BoundedLine {
    bytes_read: u64,
    preview: Vec<u8>,
    oversized: bool,
}

fn read_bounded_line(reader: &mut impl BufRead, max_bytes: usize) -> Result<BoundedLine> {
    let mut bytes_read = 0_u64;
    let mut preview = Vec::with_capacity(max_bytes.min(8192));
    let mut terminated = false;

    while !terminated {
        let available = reader.fill_buf().context("read collector source")?;
        if available.is_empty() {
            break;
        }
        let consumed = available
            .iter()
            .position(|byte| *byte == b'\n')
            .map(|position| position + 1)
            .unwrap_or(available.len());
        let remaining = max_bytes.saturating_add(1).saturating_sub(preview.len());
        preview.extend_from_slice(&available[..consumed.min(remaining)]);
        bytes_read = bytes_read
            .checked_add(consumed as u64)
            .context("collector line byte count overflow")?;
        terminated = available[..consumed].last() == Some(&b'\n');
        reader.consume(consumed);
    }

    Ok(BoundedLine {
        bytes_read,
        oversized: bytes_read > max_bytes as u64,
        preview,
    })
}

fn discard_acknowledged(reader: &mut impl Read, mut bytes: u64) -> Result<()> {
    let mut buffer = [0_u8; 8192];
    while bytes > 0 {
        let wanted = usize::try_from(bytes.min(buffer.len() as u64)).unwrap();
        let read = reader
            .read(&mut buffer[..wanted])
            .context("discard acknowledged stdin bytes")?;
        if read == 0 {
            bail!(
                "stdin ended before collector checkpoint offset; replay the same source or use a new source_id"
            );
        }
        bytes -= read as u64;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use std::time::Duration;

    use super::*;
    use crate::collector::{DEFAULT_BATCH_SIZE, DEFAULT_MAX_LINE_BYTES, DEFAULT_MAX_RETRIES};

    #[test]
    fn bounded_reader_discards_oversized_line_and_continues() {
        let source = format!("{}\n{{}}\n", "x".repeat(64));
        let mut reader = Cursor::new(source.into_bytes());
        let oversized = read_bounded_line(&mut reader, 16).unwrap();
        assert!(oversized.oversized);
        assert_eq!(oversized.preview.len(), 17);
        let next = read_bounded_line(&mut reader, 16).unwrap();
        assert!(!next.oversized);
        assert_eq!(next.preview, b"{}\n");
    }

    #[test]
    fn stdin_resume_discards_exact_checkpoint_bytes() {
        let mut source = Cursor::new(b"first\nsecond\n".to_vec());
        discard_acknowledged(&mut source, 6).unwrap();
        let mut rest = String::new();
        source.read_to_string(&mut rest).unwrap();
        assert_eq!(rest, "second\n");
        assert!(discard_acknowledged(&mut Cursor::new(b"short".to_vec()), 9).is_err());
    }

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

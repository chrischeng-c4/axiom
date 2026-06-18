// SPEC-MANAGED: projects/relay/tech-design/logic/core-durable-log-single-multi-broadcast-delivery-model.md#logic
// HANDWRITE-BEGIN gap="missing-generator:logic:3e8f9afa" tracker="pending-tracker" reason="Durable ordered log substrate: append with deterministic-id dedupe, monotonic seq, RAM ring + disk segment persistence, ordered read/replay."
//! Durable ordered log substrate for one `(subject, shard)`.
//!
//! Append assigns a monotonic, gap-free [`Seq`], dedupes on [`MessageId`] for
//! idempotent at-least-once semantics, keeps the entries resident in RAM for
//! low-latency fan-out / replay, and (when a data directory is configured)
//! persists them as newline-delimited JSON that is replayed on open.

use std::collections::HashMap;
use std::fs::{create_dir_all, File, OpenOptions};
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;

use chrono::{DateTime, Utc};

use crate::config::{FsyncPolicy, RelayCoreConfig};
use crate::types::{AppendOutcome, LogEntry, MessageId, Payload, Seq, ShardId, Subject};

/// A durable ordered log for a single `(subject, shard)`.
///
/// @spec projects/relay/tech-design/logic/core-durable-log-single-multi-broadcast-delivery-model.md#logic
pub struct Log {
    subject: Subject,
    shard: ShardId,
    entries: Vec<LogEntry>,
    dedupe: HashMap<MessageId, Seq>,
    writer: Option<BufWriter<File>>,
    fsync: FsyncPolicy,
}

fn sanitize(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '.' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

impl Log {
    /// Open (and recover) the log for `(subject, shard)`.
    ///
    /// With an empty `config.data_dir` the log is RAM-only. Otherwise existing
    /// segment contents are replayed into memory and further appends are
    /// persisted.
    pub fn open(config: &RelayCoreConfig, subject: &str, shard: ShardId) -> io::Result<Log> {
        let mut log = Log {
            subject: subject.to_string(),
            shard,
            entries: Vec::new(),
            dedupe: HashMap::new(),
            writer: None,
            fsync: config.fsync,
        };

        if config.data_dir.is_empty() {
            return Ok(log);
        }

        let dir = PathBuf::from(&config.data_dir);
        create_dir_all(&dir)?;
        let path = dir.join(format!("{}__shard{}.ndjson", sanitize(subject), shard));

        if path.exists() {
            let file = File::open(&path)?;
            for line in BufReader::new(file).lines() {
                let line = line?;
                if line.trim().is_empty() {
                    continue;
                }
                let entry: LogEntry = serde_json::from_str(&line)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                log.dedupe.insert(entry.message_id.clone(), entry.seq);
                log.entries.push(entry);
            }
        }

        let file = OpenOptions::new().create(true).append(true).open(&path)?;
        log.writer = Some(BufWriter::new(file));
        Ok(log)
    }

    /// Number of entries currently in the log; also the next seq to assign.
    pub fn len(&self) -> Seq {
        self.entries.len() as Seq
    }

    /// True when the log holds no entries.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Read one entry by seq.
    pub fn entry(&self, seq: Seq) -> Option<&LogEntry> {
        self.entries.get(seq as usize)
    }

    /// Ordered view of entries from `from_seq` onward (for fan-out / replay).
    pub fn range(&self, from_seq: Seq) -> &[LogEntry] {
        let start = (from_seq as usize).min(self.entries.len());
        &self.entries[start..]
    }

    /// Append `payload` under `message_id`. Idempotent: a repeated id returns
    /// the existing seq with `deduped = true` and writes nothing new.
    ///
    /// @spec projects/relay/tech-design/logic/core-durable-log-single-multi-broadcast-delivery-model.md#logic
    pub fn append(
        &mut self,
        message_id: &str,
        payload: Payload,
        headers: std::collections::BTreeMap<String, String>,
        now: DateTime<Utc>,
    ) -> io::Result<AppendOutcome> {
        if let Some(&seq) = self.dedupe.get(message_id) {
            return Ok(AppendOutcome { seq, deduped: true });
        }

        let seq = self.entries.len() as Seq;
        let entry = LogEntry {
            seq,
            message_id: message_id.to_string(),
            subject: self.subject.clone(),
            shard: self.shard,
            payload,
            headers,
            appended_at: now,
        };

        if let Some(writer) = self.writer.as_mut() {
            let line = serde_json::to_string(&entry)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            writer.write_all(line.as_bytes())?;
            writer.write_all(b"\n")?;
            match self.fsync {
                FsyncPolicy::Always => {
                    writer.flush()?;
                    writer.get_ref().sync_all()?;
                }
                FsyncPolicy::Interval => writer.flush()?,
                FsyncPolicy::Os => {}
            }
        }

        self.dedupe.insert(message_id.to_string(), seq);
        self.entries.push(entry);
        Ok(AppendOutcome {
            seq,
            deduped: false,
        })
    }
}
// HANDWRITE-END

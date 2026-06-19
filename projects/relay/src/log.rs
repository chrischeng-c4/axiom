// SPEC-MANAGED: projects/relay/tech-design/logic/core-durable-log-single-multi-broadcast-delivery-model.md#logic
// HANDWRITE-BEGIN gap="missing-generator:logic:3e8f9afa" tracker="pending-tracker" reason="Durable ordered log substrate: append with deterministic-id dedupe, monotonic seq, RAM ring + disk segment persistence, ordered read/replay."
//! Durable ordered log substrate for one `(subject, shard)`.
//!
//! Append assigns a monotonic, gap-free [`Seq`], dedupes on [`MessageId`] for
//! idempotent at-least-once semantics, and persists entries as newline-delimited
//! JSON replayed on open.
//!
//! RAM is bounded (#130): only the most recent `ram_ring_entries` entries stay
//! resident in a ring; older entries are evicted (they remain on disk) and read
//! back on demand via a dense byte-offset index, so the log scales far beyond
//! memory. The dedupe map is likewise capped to `dedupe.window_entries`.

use std::collections::{BTreeMap, HashMap, VecDeque};
use std::fs::{create_dir_all, File, OpenOptions};
use std::io::{self, BufRead, BufReader, BufWriter, Seek, SeekFrom, Write};
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
    fsync: FsyncPolicy,
    /// Total entries appended (also the next seq to assign).
    len: Seq,
    /// Resident window: the most recent entries (at most `ram_cap` when disk-backed).
    ring: VecDeque<LogEntry>,
    /// Max resident entries (`ram_ring_entries`); only enforced when disk-backed.
    ram_cap: usize,
    /// Dense byte offset of each seq's NDJSON line (for cold disk reads).
    offsets: Vec<u64>,
    writer: Option<BufWriter<File>>,
    /// Next byte offset to write (== current file length).
    write_pos: u64,
    /// NDJSON path, for reading evicted entries back.
    read_path: Option<PathBuf>,
    dedupe: HashMap<MessageId, Seq>,
    /// FIFO of dedupe ids, to evict the oldest beyond the window.
    dedupe_order: VecDeque<MessageId>,
    dedupe_cap: usize,
    /// Sidecar path for the durable committed watermark (None = RAM-only).
    commit_path: Option<PathBuf>,
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

fn cap_or_unbounded(n: u64) -> usize {
    if n == 0 {
        usize::MAX
    } else {
        n as usize
    }
}

impl Log {
    /// Open (and recover) the log for `(subject, shard)`.
    ///
    /// With an empty `config.data_dir` the log is RAM-only (never evicts).
    /// Otherwise existing segment contents are replayed — rebuilding the offset
    /// index, the resident ring (last `ram_ring_entries`), the dedupe window,
    /// and the write position — and further appends are persisted.
    pub fn open(config: &RelayCoreConfig, subject: &str, shard: ShardId) -> io::Result<Log> {
        let mut log = Log {
            subject: subject.to_string(),
            shard,
            fsync: config.fsync,
            len: 0,
            ring: VecDeque::new(),
            ram_cap: cap_or_unbounded(config.ram_ring_entries),
            offsets: Vec::new(),
            writer: None,
            write_pos: 0,
            read_path: None,
            dedupe: HashMap::new(),
            dedupe_order: VecDeque::new(),
            dedupe_cap: cap_or_unbounded(config.dedupe.window_entries),
            commit_path: None,
        };

        if config.data_dir.is_empty() {
            return Ok(log);
        }

        let dir = PathBuf::from(&config.data_dir);
        create_dir_all(&dir)?;
        let path = dir.join(format!("{}__shard{}.ndjson", sanitize(subject), shard));
        log.read_path = Some(path.clone());
        log.commit_path = Some(dir.join(format!("{}__shard{}.commit", sanitize(subject), shard)));

        if path.exists() {
            let mut reader = BufReader::new(File::open(&path)?);
            let mut pos: u64 = 0;
            let mut line = String::new();
            loop {
                line.clear();
                let n = reader.read_line(&mut line)?;
                if n == 0 {
                    break;
                }
                let raw = line.trim_end();
                if !raw.is_empty() {
                    let entry: LogEntry = serde_json::from_str(raw)
                        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                    let seq = log.len;
                    log.offsets.push(pos);
                    log.dedupe_insert(entry.message_id.clone(), seq);
                    log.ring_push(entry, true);
                    log.len += 1;
                }
                pos += n as u64;
            }
            log.write_pos = pos;
        }

        let file = OpenOptions::new().create(true).append(true).open(&path)?;
        log.writer = Some(BufWriter::new(file));
        Ok(log)
    }

    /// Number of entries in the log; also the next seq to assign.
    pub fn len(&self) -> Seq {
        self.len
    }

    /// True when the log holds no entries.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Seq of the oldest entry still resident in the RAM ring.
    fn ring_start(&self) -> Seq {
        self.len - self.ring.len() as Seq
    }

    /// Insert an id into the bounded dedupe window, evicting the oldest beyond `dedupe_cap`.
    fn dedupe_insert(&mut self, id: MessageId, seq: Seq) {
        self.dedupe.insert(id.clone(), seq);
        self.dedupe_order.push_back(id);
        // FIFO window: dedupe and dedupe_order stay in lockstep (an id is only
        // inserted once — repeats are deduped before reaching here).
        while self.dedupe_order.len() > self.dedupe_cap {
            if let Some(old) = self.dedupe_order.pop_front() {
                self.dedupe.remove(&old);
            }
        }
    }

    /// Push an entry into the ring; when disk-backed, evict the oldest beyond `ram_cap`.
    fn ring_push(&mut self, entry: LogEntry, disk_backed: bool) {
        self.ring.push_back(entry);
        if disk_backed {
            while self.ring.len() > self.ram_cap {
                self.ring.pop_front();
            }
        }
    }

    /// Read one entry by seq — from the RAM ring when resident, else from disk
    /// via the offset index.
    ///
    /// @spec projects/relay/tech-design/logic/bounded-ram-durable-log-entry-eviction-offset-index-disk-backed.md#logic
    pub fn entry(&self, seq: Seq) -> io::Result<Option<LogEntry>> {
        if seq >= self.len {
            return Ok(None);
        }
        let start = self.ring_start();
        if seq >= start {
            return Ok(Some(self.ring[(seq - start) as usize].clone()));
        }
        Ok(self.read_disk_range(seq, seq + 1)?.into_iter().next())
    }

    /// Ordered entries from `from_seq` onward (for fan-out / replay): the cold
    /// (evicted) prefix is read sequentially from disk, the hot tail from the ring.
    ///
    /// @spec projects/relay/tech-design/logic/bounded-ram-durable-log-entry-eviction-offset-index-disk-backed.md#logic
    pub fn range(&self, from_seq: Seq) -> io::Result<Vec<LogEntry>> {
        let from = from_seq.min(self.len);
        let start = self.ring_start();
        let mut out = Vec::with_capacity((self.len - from) as usize);
        if from < start {
            out.extend(self.read_disk_range(from, start)?);
        }
        let hot_from = from.max(start);
        for seq in hot_from..self.len {
            out.push(self.ring[(seq - start) as usize].clone());
        }
        Ok(out)
    }

    /// Read seqs `[from, to)` back from the on-disk NDJSON via the offset index.
    fn read_disk_range(&self, from: Seq, to: Seq) -> io::Result<Vec<LogEntry>> {
        let path = self
            .read_path
            .as_ref()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no on-disk segment"))?;
        let mut reader = BufReader::new(File::open(path)?);
        reader.seek(SeekFrom::Start(self.offsets[from as usize]))?;
        let mut out = Vec::with_capacity((to - from) as usize);
        let mut line = String::new();
        for _ in from..to {
            line.clear();
            if reader.read_line(&mut line)? == 0 {
                break;
            }
            let entry: LogEntry = serde_json::from_str(line.trim_end())
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            out.push(entry);
        }
        Ok(out)
    }

    /// Write one entry's NDJSON line, recording its byte offset; respects the
    /// per-append fsync policy.
    fn write_entry(&mut self, entry: &LogEntry) -> io::Result<()> {
        let offset = self.write_pos;
        if let Some(writer) = self.writer.as_mut() {
            let line = serde_json::to_string(entry)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            writer.write_all(line.as_bytes())?;
            writer.write_all(b"\n")?;
            self.write_pos += line.len() as u64 + 1;
            match self.fsync {
                FsyncPolicy::Always => {
                    writer.flush()?;
                    writer.get_ref().sync_all()?;
                }
                FsyncPolicy::Interval => writer.flush()?,
                FsyncPolicy::Os => {}
            }
        }
        self.offsets.push(offset);
        Ok(())
    }

    /// Append `payload` under `message_id`. Idempotent: a repeated id returns
    /// the existing seq with `deduped = true` and writes nothing new.
    ///
    /// @spec projects/relay/tech-design/logic/core-durable-log-single-multi-broadcast-delivery-model.md#logic
    pub fn append(
        &mut self,
        message_id: &str,
        payload: Payload,
        headers: BTreeMap<String, String>,
        now: DateTime<Utc>,
    ) -> io::Result<AppendOutcome> {
        if let Some(&seq) = self.dedupe.get(message_id) {
            return Ok(AppendOutcome { seq, deduped: true });
        }
        let seq = self.len;
        let entry = LogEntry {
            seq,
            message_id: message_id.to_string(),
            subject: self.subject.clone(),
            shard: self.shard,
            payload,
            headers,
            appended_at: now,
        };
        self.write_entry(&entry)?;
        let disk = self.writer.is_some();
        self.dedupe_insert(message_id.to_string(), seq);
        self.ring_push(entry, disk);
        self.len += 1;
        Ok(AppendOutcome {
            seq,
            deduped: false,
        })
    }

    /// Append a batch, then issue ONE `sync_all` for the whole batch (group
    /// commit): every entry becomes durable with a single fsync, so durability
    /// cost is amortized. Idempotent per `message_id` (against existing entries
    /// and within the batch). Returns one outcome per input, in order.
    ///
    /// @spec projects/relay/tech-design/logic/default-durable-engine-throughput-group-commit-fsync-publish-bat.md#logic
    pub fn append_many(
        &mut self,
        items: Vec<(String, Payload, BTreeMap<String, String>)>,
        now: DateTime<Utc>,
    ) -> io::Result<Vec<AppendOutcome>> {
        let mut outcomes = Vec::with_capacity(items.len());
        let disk = self.writer.is_some();

        for (message_id, payload, headers) in items {
            if let Some(&seq) = self.dedupe.get(&message_id) {
                outcomes.push(AppendOutcome { seq, deduped: true });
                continue;
            }
            let seq = self.len;
            let entry = LogEntry {
                seq,
                message_id: message_id.clone(),
                subject: self.subject.clone(),
                shard: self.shard,
                payload,
                headers,
                appended_at: now,
            };
            // Write the line + record the offset, but defer the fsync (group commit).
            let offset = self.write_pos;
            if let Some(writer) = self.writer.as_mut() {
                let line = serde_json::to_string(&entry)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                writer.write_all(line.as_bytes())?;
                writer.write_all(b"\n")?;
                self.write_pos += line.len() as u64 + 1;
            }
            self.offsets.push(offset);
            self.dedupe_insert(message_id, seq);
            self.ring_push(entry, disk);
            self.len += 1;
            outcomes.push(AppendOutcome {
                seq,
                deduped: false,
            });
        }

        // Group commit: a single fsync makes the whole batch durable.
        if let Some(writer) = self.writer.as_mut() {
            writer.flush()?;
            writer.get_ref().sync_all()?;
        }
        Ok(outcomes)
    }

    /// Durably record the committed watermark (one write + fsync). No-op when
    /// RAM-only.
    ///
    /// @spec projects/relay/tech-design/logic/default-durable-engine-throughput-group-commit-fsync-publish-bat.md#logic
    pub fn persist_commit(&self, watermark: Seq) -> io::Result<()> {
        if let Some(path) = &self.commit_path {
            let mut f = File::create(path)?;
            f.write_all(watermark.to_string().as_bytes())?;
            f.sync_all()?;
        }
        Ok(())
    }

    /// Load the durable committed watermark recorded by a previous run, if any.
    ///
    /// @spec projects/relay/tech-design/logic/default-durable-engine-throughput-group-commit-fsync-publish-bat.md#logic
    pub fn load_commit(&self) -> Option<Seq> {
        let path = self.commit_path.as_ref()?;
        std::fs::read_to_string(path).ok()?.trim().parse().ok()
    }
}
// HANDWRITE-END

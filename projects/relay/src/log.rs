// SPEC-MANAGED: projects/relay/tech-design/logic/core-durable-log-single-multi-broadcast-delivery-model.md#logic
// HANDWRITE-BEGIN gap="missing-generator:logic:3e8f9afa" tracker="pending-tracker" reason="Durable ordered log substrate: append with deterministic-id dedupe, monotonic seq, RAM ring + disk segment persistence, ordered read/replay."
//! Durable ordered log substrate for one `(subject, shard)`.
//!
//! Append assigns a monotonic, gap-free [`Seq`], dedupes on [`MessageId`], and
//! persists entries as newline-delimited JSON. Storage is **segmented** (#131):
//! the active segment rolls at `segment_bytes`, and retention prunes the oldest
//! whole segments by `max_bytes_per_shard` / `max_age_secs`, advancing
//! `start_seq`. RAM is bounded (#130): only the most recent `ram_ring_entries`
//! stay resident; older (but un-pruned) entries are read back from their segment
//! via a per-segment byte-offset index. The dedupe map is a FIFO window.

use std::collections::{BTreeMap, HashMap, VecDeque};
use std::fs::{create_dir_all, File, OpenOptions};
use std::io::{self, BufRead, BufReader, BufWriter, Seek, SeekFrom, Write};
use std::path::PathBuf;

use chrono::{DateTime, Duration, Utc};

use crate::config::{FsyncPolicy, RelayCoreConfig};
use crate::types::{AppendOutcome, LogEntry, MessageId, Payload, Seq, ShardId, Subject};

/// One on-disk NDJSON segment holding seqs `[base_seq, next segment's base_seq)`.
struct Segment {
    base_seq: Seq,
    path: PathBuf,
    bytes: u64,
    last_ts: DateTime<Utc>,
}

/// A durable ordered log for a single `(subject, shard)`.
///
/// @spec projects/relay/tech-design/logic/core-durable-log-single-multi-broadcast-delivery-model.md#logic
pub struct Log {
    subject: Subject,
    shard: ShardId,
    fsync: FsyncPolicy,
    /// Total entries ever appended (also the next seq to assign).
    len: Seq,
    /// Earliest still-available seq (advances as old segments are pruned).
    start_seq: Seq,
    /// Resident window: the most recent entries (at most `ram_cap` when disk-backed).
    ring: VecDeque<LogEntry>,
    ram_cap: usize,
    /// Sparse byte-offset index: sorted (seq, offset-within-segment), sampled
    /// every INDEX_STRIDE entries plus an anchor at each segment's base_seq.
    index: Vec<(Seq, u64)>,
    dedupe: HashMap<MessageId, Seq>,
    dedupe_order: VecDeque<MessageId>,
    dedupe_cap: usize,
    // ---- disk (None fields => RAM-only) ----
    dir: Option<PathBuf>,
    segment_bytes: u64,
    max_bytes: u64,
    max_age: i64,
    segments: Vec<Segment>,
    writer: Option<BufWriter<File>>,
    /// Bytes in the active (last) segment.
    active_bytes: u64,
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

/// Index one byte offset every this many entries (plus a per-segment anchor),
/// so the in-RAM index is ~len/INDEX_STRIDE points instead of one per seq.
const INDEX_STRIDE: Seq = 64;

/// Whether `seq` is sampled into the sparse index for a segment based at `base`.
fn is_indexed(seq: Seq, base: Seq) -> bool {
    seq == base || (seq - base) % INDEX_STRIDE == 0
}

impl Log {
    /// Open (and recover) the log for `(subject, shard)`.
    ///
    /// With an empty `config.data_dir` the log is RAM-only. Otherwise all
    /// surviving segment files are replayed in seq order — rebuilding the
    /// segment list, the per-segment offset index, the resident ring, the
    /// dedupe window, `len` and `start_seq` — and the last segment is reopened
    /// for appends.
    pub fn open(config: &RelayCoreConfig, subject: &str, shard: ShardId) -> io::Result<Log> {
        let mut log = Log {
            subject: subject.to_string(),
            shard,
            fsync: config.fsync,
            len: 0,
            start_seq: 0,
            ring: VecDeque::new(),
            ram_cap: cap_or_unbounded(config.ram_ring_entries),
            index: Vec::new(),
            dedupe: HashMap::new(),
            dedupe_order: VecDeque::new(),
            dedupe_cap: cap_or_unbounded(config.dedupe.window_entries),
            dir: None,
            segment_bytes: config.segment_bytes.max(1),
            max_bytes: config.retention.max_bytes_per_shard,
            max_age: config.retention.max_age_secs as i64,
            segments: Vec::new(),
            writer: None,
            active_bytes: 0,
            commit_path: None,
        };

        if config.data_dir.is_empty() {
            return Ok(log);
        }

        let dir = PathBuf::from(&config.data_dir);
        create_dir_all(&dir)?;
        log.commit_path = Some(dir.join(format!("{}__shard{}.commit", sanitize(subject), shard)));

        // Discover existing segment files: <subject>__shardN__<base>.ndjson
        let prefix = format!("{}__shard{}__", sanitize(subject), shard);
        let mut found: Vec<(Seq, PathBuf)> = Vec::new();
        for ent in std::fs::read_dir(&dir)? {
            let path = ent?.path();
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if let Some(rest) = name.strip_prefix(&prefix) {
                    if let Some(base) = rest.strip_suffix(".ndjson") {
                        if let Ok(b) = base.parse::<Seq>() {
                            found.push((b, path));
                        }
                    }
                }
            }
        }
        found.sort_by_key(|(b, _)| *b);

        // Seqs and the offset index are relative to the earliest surviving seq.
        let base0 = found.first().map(|(b, _)| *b).unwrap_or(0);
        log.start_seq = base0;
        log.len = base0;

        for (base, path) in &found {
            let mut reader = BufReader::new(File::open(path)?);
            let mut pos: u64 = 0;
            let mut line = String::new();
            let mut seg = Segment {
                base_seq: *base,
                path: path.clone(),
                bytes: 0,
                last_ts: Utc::now(),
            };
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
                    if is_indexed(seq, *base) {
                        log.index.push((seq, pos));
                    }
                    seg.last_ts = entry.appended_at;
                    log.dedupe_insert(entry.message_id.clone(), seq);
                    log.ring_push(entry, true);
                    log.len += 1;
                }
                pos += n as u64;
            }
            seg.bytes = pos;
            log.segments.push(seg);
        }
        log.start_seq = log.segments.first().map(|s| s.base_seq).unwrap_or(0);
        log.dir = Some(dir);

        // Open (or create) the active segment for appends.
        if log.segments.is_empty() {
            log.start_new_segment(0)?;
        } else {
            let active = log.segments.last().unwrap();
            log.active_bytes = active.bytes;
            log.writer = Some(open_append(&active.path)?);
        }
        Ok(log)
    }

    fn segment_path(&self, base: Seq) -> PathBuf {
        self.dir.as_ref().unwrap().join(format!(
            "{}__shard{}__{}.ndjson",
            sanitize(&self.subject),
            self.shard,
            base
        ))
    }

    /// Close the active segment (if any) and open a fresh one at `base`.
    fn start_new_segment(&mut self, base: Seq) -> io::Result<()> {
        if let Some(w) = self.writer.as_mut() {
            w.flush()?;
            w.get_ref().sync_all()?;
        }
        let path = self.segment_path(base);
        self.writer = Some(open_append(&path)?);
        self.active_bytes = 0;
        self.segments.push(Segment {
            base_seq: base,
            path,
            bytes: 0,
            last_ts: Utc::now(),
        });
        Ok(())
    }

    pub fn len(&self) -> Seq {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Earliest still-available seq (entries below it have been pruned).
    pub fn start_seq(&self) -> Seq {
        self.start_seq
    }

    /// Number of sparse index points (≈ len / INDEX_STRIDE). For tests / metrics.
    ///
    /// @spec projects/relay/tech-design/logic/sparse-offset-index-scale-the-log-to-billions-of-entries.md#logic
    pub fn index_entries(&self) -> usize {
        self.index.len()
    }

    fn ring_start(&self) -> Seq {
        self.len - self.ring.len() as Seq
    }

    fn dedupe_insert(&mut self, id: MessageId, seq: Seq) {
        self.dedupe.insert(id.clone(), seq);
        self.dedupe_order.push_back(id);
        while self.dedupe_order.len() > self.dedupe_cap {
            if let Some(old) = self.dedupe_order.pop_front() {
                self.dedupe.remove(&old);
            }
        }
    }

    fn ring_push(&mut self, entry: LogEntry, disk_backed: bool) {
        self.ring.push_back(entry);
        if disk_backed {
            while self.ring.len() > self.ram_cap {
                self.ring.pop_front();
            }
        }
    }

    /// Index of the segment holding `seq` (largest base_seq <= seq).
    fn segment_for(&self, seq: Seq) -> usize {
        self.segments.partition_point(|s| s.base_seq <= seq) - 1
    }

    /// Exclusive end seq of segment `i`.
    fn segment_end(&self, i: usize) -> Seq {
        self.segments
            .get(i + 1)
            .map(|s| s.base_seq)
            .unwrap_or(self.len)
    }

    /// Read one entry by seq — RAM ring when resident, else its segment on disk;
    /// `None` if out of range or pruned.
    ///
    /// @spec projects/relay/tech-design/logic/bounded-ram-durable-log-entry-eviction-offset-index-disk-backed.md#logic
    pub fn entry(&self, seq: Seq) -> io::Result<Option<LogEntry>> {
        if seq >= self.len || seq < self.start_seq {
            return Ok(None);
        }
        let start = self.ring_start();
        if seq >= start {
            return Ok(Some(self.ring[(seq - start) as usize].clone()));
        }
        Ok(self.read_disk_range(seq, seq + 1)?.into_iter().next())
    }

    /// Ordered entries from `from_seq` (clamped up to `start_seq`) onward; the
    /// cold prefix is read from disk segments in order, the hot tail from RAM.
    ///
    /// @spec projects/relay/tech-design/logic/log-segment-rotation-retention-full-log-lifecycle.md#logic
    pub fn range(&self, from_seq: Seq) -> io::Result<Vec<LogEntry>> {
        let from = from_seq.max(self.start_seq).min(self.len);
        let ring_start = self.ring_start();
        let mut out = Vec::with_capacity((self.len - from) as usize);
        if from < ring_start {
            out.extend(self.read_disk_range(from, ring_start)?);
        }
        for seq in from.max(ring_start)..self.len {
            out.push(self.ring[(seq - ring_start) as usize].clone());
        }
        Ok(out)
    }

    /// Read seqs `[from, to)` from disk, walking each segment's run in order.
    fn read_disk_range(&self, from: Seq, to: Seq) -> io::Result<Vec<LogEntry>> {
        if self.dir.is_none() {
            return Ok(Vec::new());
        }
        let mut out = Vec::with_capacity((to - from) as usize);
        let mut seq = from;
        while seq < to {
            let si = self.segment_for(seq);
            let run_end = self.segment_end(si).min(to);
            // Nearest sparse index point with index_seq <= seq. The per-segment
            // anchor guarantees one exists in this seq's own segment.
            let ip = self.index.partition_point(|&(s, _)| s <= seq) - 1;
            let (idx_seq, idx_off) = self.index[ip];
            let mut reader = BufReader::new(File::open(&self.segments[si].path)?);
            reader.seek(SeekFrom::Start(idx_off))?;
            let mut line = String::new();
            // Scan forward from the indexed point to `seq`.
            for _ in idx_seq..seq {
                line.clear();
                if reader.read_line(&mut line)? == 0 {
                    break;
                }
            }
            for _ in seq..run_end {
                line.clear();
                if reader.read_line(&mut line)? == 0 {
                    break;
                }
                let entry: LogEntry = serde_json::from_str(line.trim_end())
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                out.push(entry);
            }
            seq = run_end;
        }
        Ok(out)
    }

    /// Roll to a new segment when the active one is full.
    fn rotate_if_needed(&mut self) -> io::Result<()> {
        if self.dir.is_some() && self.active_bytes >= self.segment_bytes {
            // active segment has at least one entry (base_seq < len) once full.
            let base = self.len;
            self.start_new_segment(base)?;
        }
        Ok(())
    }

    /// Write one entry's line to the active segment, recording its in-segment offset.
    fn write_line(&mut self, entry: &LogEntry) -> io::Result<()> {
        let offset = self.active_bytes;
        if let Some(writer) = self.writer.as_mut() {
            let line = serde_json::to_string(entry)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            writer.write_all(line.as_bytes())?;
            writer.write_all(b"\n")?;
            self.active_bytes += line.len() as u64 + 1;
            if let Some(seg) = self.segments.last_mut() {
                seg.bytes = self.active_bytes;
                seg.last_ts = entry.appended_at;
            }
        }
        let base = self.segments.last().map(|s| s.base_seq).unwrap_or(0);
        if is_indexed(entry.seq, base) {
            self.index.push((entry.seq, offset));
        }
        Ok(())
    }

    fn fsync_active(&mut self) -> io::Result<()> {
        if let Some(writer) = self.writer.as_mut() {
            writer.flush()?;
            writer.get_ref().sync_all()?;
        }
        Ok(())
    }

    /// Prune the oldest whole segments by total bytes / age, advancing `start_seq`.
    /// Never prunes the active (last) segment.
    fn prune(&mut self, now: DateTime<Utc>) -> io::Result<()> {
        loop {
            if self.segments.len() <= 1 {
                break;
            }
            let total: u64 = self.segments.iter().map(|s| s.bytes).sum();
            let oldest = &self.segments[0];
            let over_bytes = self.max_bytes > 0 && total > self.max_bytes;
            let too_old =
                self.max_age > 0 && oldest.last_ts < now - Duration::seconds(self.max_age);
            if !over_bytes && !too_old {
                break;
            }
            let removed = self.segments.remove(0);
            let _ = std::fs::remove_file(&removed.path);
            let new_start = self.segments[0].base_seq;
            // Drop sparse index points below the new start.
            let drop_n = self.index.partition_point(|&(s, _)| s < new_start);
            self.index.drain(0..drop_n);
            self.start_seq = new_start;
        }
        Ok(())
    }

    /// Append `payload` under `message_id`. Idempotent on `message_id`.
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
        self.rotate_if_needed()?;
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
        self.write_line(&entry)?;
        match self.fsync {
            FsyncPolicy::Always => self.fsync_active()?,
            FsyncPolicy::Interval => {
                if let Some(w) = self.writer.as_mut() {
                    w.flush()?;
                }
            }
            FsyncPolicy::Os => {}
        }
        let disk = self.writer.is_some();
        self.dedupe_insert(message_id.to_string(), seq);
        self.ring_push(entry, disk);
        self.len += 1;
        self.prune(now)?;
        Ok(AppendOutcome {
            seq,
            deduped: false,
        })
    }

    /// Append a batch with ONE fsync (group commit). Idempotent per `message_id`
    /// (against existing entries and within the batch).
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
            self.rotate_if_needed()?;
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
            self.write_line(&entry)?;
            self.dedupe_insert(message_id, seq);
            self.ring_push(entry, disk);
            self.len += 1;
            outcomes.push(AppendOutcome {
                seq,
                deduped: false,
            });
        }
        // Group commit: a single fsync makes the whole batch durable.
        if self.fsync != FsyncPolicy::Os {
            self.fsync_active()?;
        }
        self.prune(now)?;
        Ok(outcomes)
    }

    /// Durably record the committed watermark (one write + fsync). No-op when RAM-only.
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

fn open_append(path: &PathBuf) -> io::Result<BufWriter<File>> {
    Ok(BufWriter::new(
        OpenOptions::new().create(true).append(true).open(path)?,
    ))
}
// HANDWRITE-END

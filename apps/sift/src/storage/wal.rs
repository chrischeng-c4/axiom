use std::{
    collections::{BTreeMap, HashSet},
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    sync::Mutex,
};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

use crate::{SignalKind, StoredEvent};

use super::archive::ArchiveWatermarks;

pub struct SignalWal {
    logs: WalFile,
    metrics: WalFile,
    traces: WalFile,
}

struct WalFile {
    signal: SignalKind,
    path: PathBuf,
    writer: Mutex<storage_durable::FramedLogWriter>,
}

const WAL_BATCH_FORMAT_VERSION: u16 = 1;

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct WalBatch {
    format_version: u16,
    first_cursor: u64,
    last_cursor: u64,
    events: Vec<StoredEvent>,
}

impl SignalWal {
    pub fn open(root: impl AsRef<Path>) -> Result<Self> {
        let root = root.as_ref().join("wal");
        Ok(Self {
            logs: WalFile::open(&root, "logs", SignalKind::Log)?,
            metrics: WalFile::open(&root, "metrics", SignalKind::Metric)?,
            traces: WalFile::open(&root, "traces", SignalKind::Span)?,
        })
    }

    pub fn append(&self, event: &StoredEvent) -> Result<()> {
        self.append_batch(std::slice::from_ref(event))
    }

    /// Write one signal batch as one CRC frame and one fsync boundary.
    pub fn append_batch(&self, events: &[StoredEvent]) -> Result<()> {
        let signal = events
            .first()
            .context("signal WAL batch must not be empty")?
            .event
            .signal;
        self.file(signal)?.append_batch(events)
    }

    pub fn recovered_events(&self) -> Result<Vec<StoredEvent>> {
        let mut by_cursor = BTreeMap::<u64, StoredEvent>::new();
        let mut event_ids = HashSet::new();
        for file in [&self.logs, &self.metrics, &self.traces] {
            for event in file.recovered_events()? {
                if !event_ids.insert(event.event.event_id.clone()) {
                    bail!("WAL contains duplicate event_id {}", event.event.event_id);
                }
                if let Some(existing) = by_cursor.insert(event.cursor, event.clone()) {
                    bail!(
                        "WAL cursor {} belongs to both {} and {}",
                        event.cursor,
                        existing.event.event_id,
                        event.event.event_id
                    );
                }
            }
        }
        Ok(by_cursor.into_values().collect())
    }

    pub(crate) fn query_events(&self, after: u64, limit: usize) -> Result<Vec<StoredEvent>> {
        if limit == 0 {
            return Ok(Vec::new());
        }
        let mut events = Vec::new();
        for file in [&self.logs, &self.metrics, &self.traces] {
            events.extend(file.query_events(after, limit)?);
        }
        events.sort_by_key(|event| event.cursor);
        let mut canonical = Vec::<StoredEvent>::with_capacity(events.len());
        for event in events {
            if let Some(previous) = canonical.last() {
                if previous.cursor == event.cursor {
                    if previous != &event {
                        bail!("signal WAL files disagree at cursor {}", event.cursor);
                    }
                    continue;
                }
            }
            canonical.push(event);
        }
        canonical.truncate(limit);
        Ok(canonical)
    }

    pub(crate) fn compact_through(&self, watermarks: ArchiveWatermarks) -> Result<()> {
        for file in [&self.logs, &self.metrics, &self.traces] {
            file.compact_through(watermarks.through(file.signal))?;
        }
        Ok(())
    }

    fn file(&self, signal: SignalKind) -> Result<&WalFile> {
        match signal {
            SignalKind::Log => Ok(&self.logs),
            SignalKind::Metric => Ok(&self.metrics),
            SignalKind::Span => Ok(&self.traces),
        }
    }
}

impl WalFile {
    fn open(root: &Path, name: &str, signal: SignalKind) -> Result<Self> {
        let directory = root.join(name);
        fs::create_dir_all(&directory)
            .with_context(|| format!("create signal WAL directory {}", directory.display()))?;
        fs::set_permissions(&directory, fs::Permissions::from_mode(0o700))
            .with_context(|| format!("set signal WAL directory mode {}", directory.display()))?;
        let path = directory.join("events.framed");
        let writer =
            storage_durable::FramedLogWriter::open(&path, storage_durable::FsyncPolicy::Interval)?;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600))
            .with_context(|| format!("set signal WAL file mode {}", path.display()))?;
        Ok(Self {
            signal,
            path,
            writer: Mutex::new(writer),
        })
    }

    fn append_batch(&self, events: &[StoredEvent]) -> Result<()> {
        let first = events
            .first()
            .context("signal WAL batch must not be empty")?;
        if events.iter().any(|event| event.event.signal != self.signal) {
            bail!("event signal does not match WAL signal");
        }
        if events
            .windows(2)
            .any(|pair| pair[0].cursor.checked_add(1) != Some(pair[1].cursor))
        {
            bail!("signal WAL batch cursors must be contiguous");
        }
        let last = events.last().expect("non-empty signal WAL batch");
        let batch = WalBatch {
            format_version: WAL_BATCH_FORMAT_VERSION,
            first_cursor: first.cursor,
            last_cursor: last.cursor,
            events: events.to_vec(),
        };
        let mut writer = self.writer.lock().expect("signal WAL writer lock poisoned");
        writer
            .append(last.cursor, &serde_json::to_vec(&batch)?)
            .with_context(|| format!("append signal WAL batch {}", self.path.display()))?;
        writer
            .sync()
            .with_context(|| format!("fsync signal WAL batch {}", self.path.display()))?;
        Ok(())
    }

    fn recovered_events(&self) -> Result<Vec<StoredEvent>> {
        let mut events = Vec::new();
        for frame in storage_durable::FramedLogReader::read_frames(&self.path, 0)? {
            let batch: WalBatch = serde_json::from_slice(&frame.payload)
                .with_context(|| format!("decode signal WAL batch frame {}", frame.seq))?;
            if batch.format_version != WAL_BATCH_FORMAT_VERSION {
                bail!(
                    "unsupported signal WAL batch format {}",
                    batch.format_version
                );
            }
            let first = batch.events.first().with_context(|| {
                format!("signal WAL frame {} contains an empty batch", frame.seq)
            })?;
            let last = batch.events.last().expect("validated non-empty WAL batch");
            if batch.first_cursor != first.cursor
                || batch.last_cursor != last.cursor
                || batch.last_cursor != frame.seq
            {
                bail!(
                    "signal WAL frame sequence {} does not match batch cursors {}..{}",
                    frame.seq,
                    batch.first_cursor,
                    batch.last_cursor
                );
            }
            if batch
                .events
                .windows(2)
                .any(|pair| pair[0].cursor.checked_add(1) != Some(pair[1].cursor))
            {
                bail!("signal WAL batch cursors are not contiguous");
            }
            if batch
                .events
                .iter()
                .any(|event| event.event.signal != self.signal)
            {
                bail!(
                    "signal WAL {} contains the wrong signal",
                    self.path.display()
                );
            }
            events.extend(batch.events);
        }
        Ok(events)
    }

    fn query_events(&self, after: u64, limit: usize) -> Result<Vec<StoredEvent>> {
        let mut events = Vec::with_capacity(limit.min(1_000));
        for frame in
            storage_durable::FramedLogReader::read_frames_bounded(&self.path, after, limit)?
        {
            let batch: WalBatch = serde_json::from_slice(&frame.payload)
                .with_context(|| format!("decode signal WAL batch frame {}", frame.seq))?;
            validate_batch(self.signal, &self.path, frame.seq, &batch)?;
            for event in batch
                .events
                .into_iter()
                .filter(|event| event.cursor > after)
            {
                events.push(event);
                if events.len() == limit {
                    return Ok(events);
                }
            }
        }
        Ok(events)
    }

    fn compact_through(&self, cursor: u64) -> Result<()> {
        self.writer
            .lock()
            .expect("signal WAL writer lock poisoned")
            .truncate_through(cursor)
            .with_context(|| {
                format!(
                    "compact signal WAL {} through committed archive cursor {cursor}",
                    self.path.display()
                )
            })?;
        fs::set_permissions(&self.path, fs::Permissions::from_mode(0o600))?;
        Ok(())
    }
}

fn validate_batch(
    signal: SignalKind,
    path: &Path,
    frame_sequence: u64,
    batch: &WalBatch,
) -> Result<()> {
    if batch.format_version != WAL_BATCH_FORMAT_VERSION {
        bail!(
            "unsupported signal WAL batch format {}",
            batch.format_version
        );
    }
    let first = batch
        .events
        .first()
        .with_context(|| format!("signal WAL frame {frame_sequence} contains an empty batch"))?;
    let last = batch.events.last().expect("validated non-empty WAL batch");
    if batch.first_cursor != first.cursor
        || batch.last_cursor != last.cursor
        || batch.last_cursor != frame_sequence
    {
        bail!(
            "signal WAL frame sequence {frame_sequence} does not match batch cursors {}..{}",
            batch.first_cursor,
            batch.last_cursor
        );
    }
    if batch
        .events
        .windows(2)
        .any(|pair| pair[0].cursor.checked_add(1) != Some(pair[1].cursor))
    {
        bail!("signal WAL batch cursors are not contiguous");
    }
    if batch
        .events
        .iter()
        .any(|event| event.event.signal != signal)
    {
        bail!("signal WAL {} contains the wrong signal", path.display());
    }
    Ok(())
}

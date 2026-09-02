use std::{
    collections::{BTreeMap, HashSet, VecDeque},
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

/// Stateful, globally ordered reader over the three signal WAL files.
///
/// Each file keeps one decoded Raft batch at most. Repeated pages continue
/// from the current byte offsets instead of rescanning the WAL prefix.
pub struct SignalWalReader {
    streams: Vec<WalEventStream>,
    peeked: Vec<Option<StoredEvent>>,
}

struct WalEventStream {
    signal: SignalKind,
    path: PathBuf,
    frames: storage_durable::FramedLogCursor,
    events: VecDeque<StoredEvent>,
    after: u64,
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

    #[doc(hidden)]
    pub fn reader(&self, after: u64) -> Result<SignalWalReader> {
        let streams = [&self.logs, &self.metrics, &self.traces]
            .into_iter()
            .map(|file| WalEventStream::open(file.signal, file.path.clone(), after))
            .collect::<Result<Vec<_>>>()?;
        Ok(SignalWalReader {
            peeked: vec![None; streams.len()],
            streams,
        })
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

impl SignalWalReader {
    /// Read at most `item_limit` events and approximately `byte_limit` encoded
    /// bytes. One event is always allowed so a large valid record can make
    /// progress.
    pub fn read_page(&mut self, item_limit: usize, byte_limit: usize) -> Result<Vec<StoredEvent>> {
        if item_limit == 0 || byte_limit == 0 {
            return Ok(Vec::new());
        }
        let mut page = Vec::with_capacity(item_limit.min(1_000));
        let mut bytes = 0_usize;
        while page.len() < item_limit {
            for index in 0..self.streams.len() {
                if self.peeked[index].is_none() {
                    self.peeked[index] = self.streams[index].next_event()?;
                }
            }
            let Some((selected, cursor)) = self
                .peeked
                .iter()
                .enumerate()
                .filter_map(|(index, event)| event.as_ref().map(|event| (index, event.cursor)))
                .min_by_key(|(_, cursor)| *cursor)
            else {
                break;
            };
            let encoded = serde_json::to_vec(
                self.peeked[selected]
                    .as_ref()
                    .expect("selected WAL event exists"),
            )?
            .len();
            if !page.is_empty() && bytes.saturating_add(encoded) > byte_limit {
                break;
            }
            let event = self.peeked[selected]
                .take()
                .expect("selected WAL event exists");
            for (index, duplicate) in self.peeked.iter_mut().enumerate() {
                if index == selected || duplicate.as_ref().map(|row| row.cursor) != Some(cursor) {
                    continue;
                }
                let duplicate = duplicate.take().expect("matched WAL event exists");
                if duplicate != event {
                    bail!("signal WAL files disagree at cursor {cursor}");
                }
            }
            bytes = bytes.saturating_add(encoded);
            page.push(event);
        }
        Ok(page)
    }

    #[doc(hidden)]
    pub fn buffered_event_count_for_diagnostics(&self) -> usize {
        self.peeked.iter().filter(|event| event.is_some()).count()
            + self
                .streams
                .iter()
                .map(|stream| stream.events.len())
                .sum::<usize>()
    }
}

impl WalEventStream {
    fn open(signal: SignalKind, path: PathBuf, after: u64) -> Result<Self> {
        Ok(Self {
            signal,
            frames: storage_durable::FramedLogCursor::open(&path)?,
            path,
            events: VecDeque::new(),
            after,
        })
    }

    fn next_event(&mut self) -> Result<Option<StoredEvent>> {
        loop {
            if let Some(event) = self.events.pop_front() {
                if event.cursor > self.after {
                    self.after = event.cursor;
                    return Ok(Some(event));
                }
                continue;
            }
            let Some(frame) = self.frames.next_frame()? else {
                return Ok(None);
            };
            let batch: WalBatch = serde_json::from_slice(&frame.payload)
                .with_context(|| format!("decode signal WAL batch frame {}", frame.seq))?;
            validate_batch(self.signal, &self.path, frame.seq, &batch)?;
            self.events = batch.events.into();
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

//! Replicated command and snapshot boundary for Sift's phase-one signals.

use std::{
    collections::BTreeMap,
    fs,
    io::{Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};

use anyhow::{bail, Context, Result};
use raft_runtime::{Index, RaftStateMachine};
use serde::{Deserialize, Serialize};

use crate::{AppendResult, DurableJournal, EventEnvelope, EventQuery, StoredEvent};

const CONTROL_STATE_FILE: &str = "sift-control-state.json";
const CONTROL_STATE_FORMAT_VERSION: u16 = 1;
pub const RAFT_BATCH_MAX_BYTES: usize = 1_048_576;
pub const RAFT_BATCH_MAX_ITEMS: usize = 1_000;
pub const RAFT_BATCH_MAX_DELAY: Duration = Duration::from_millis(10);
pub const SNAPSHOT_CONTENT_TYPE: &str = "application/vnd.axiom.sift-snapshot";

const SNAPSHOT_MAGIC: &[u8; 8] = b"SIFTSNP2";
const SNAPSHOT_FORMAT_VERSION: u16 = 2;
const SNAPSHOT_HEADER_BYTES: usize = 40;
const SNAPSHOT_FRAME_HEADER_BYTES: usize = 16;
const SNAPSHOT_PAGE_EVENTS: usize = 10_000;
const MAX_SNAPSHOT_EVENT_BYTES: usize = 16 * 1024 * 1024;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum SiftCommandV1 {
    AppendEvents { events: Vec<EventEnvelope> },
}

impl SiftCommandV1 {
    pub(crate) fn encoded(&self) -> Result<Vec<u8>> {
        let bytes = serde_json::to_vec(self).context("encode Sift state-machine command")?;
        if bytes.len() > RAFT_BATCH_MAX_BYTES {
            bail!(
                "Sift Raft batch exceeds the 1 MiB limit: {} bytes",
                bytes.len()
            );
        }
        Ok(bytes)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SnapshotMetadata {
    pub applied_index: u64,
    pub last_cursor: u64,
    pub event_count: u64,
}

/// Write one stable journal prefix as framed binary data.
///
/// The writer receives one event at a time. A concurrent append can extend the
/// journal, but it cannot change the prefix captured in the header.
pub(crate) fn write_snapshot(
    journal: &DurableJournal,
    applied_index: u64,
    writer: &mut dyn Write,
) -> Result<SnapshotMetadata> {
    let (last_cursor, event_count) = journal.snapshot_bounds();
    let metadata = SnapshotMetadata {
        applied_index,
        last_cursor,
        event_count,
    };
    write_snapshot_header(writer, metadata)?;

    let mut after = 0_u64;
    let mut written = 0_u64;
    let mut expected_cursor = 1_u64;
    while written < event_count {
        let page = journal
            .query(EventQuery {
                signal: None,
                after,
                limit: SNAPSHOT_PAGE_EVENTS,
            })
            .context("read canonical journal page for snapshot")?;
        let mut progressed = false;
        for stored in page {
            if stored.cursor > last_cursor || written == event_count {
                break;
            }
            if stored.cursor != expected_cursor {
                bail!(
                    "snapshot journal cursor {} is out of order; expected {expected_cursor}",
                    stored.cursor
                );
            }
            write_snapshot_event(writer, &stored)?;
            after = stored.cursor;
            expected_cursor = expected_cursor
                .checked_add(1)
                .context("snapshot journal cursor exhausted u64")?;
            written = written
                .checked_add(1)
                .context("snapshot event count exhausted u64")?;
            progressed = true;
        }
        if !progressed {
            bail!(
                "snapshot ended after {written} events, before declared event count {event_count}"
            );
        }
    }
    if written != event_count || after != last_cursor {
        bail!(
            "snapshot prefix mismatch: wrote {written} events through cursor {after}, expected {event_count} events through cursor {last_cursor}"
        );
    }
    Ok(metadata)
}

/// Restore a seekable snapshot only after a complete validation pass.
///
/// This prevents a corrupt or truncated snapshot from partially filling an
/// empty Sift data directory.
pub(crate) fn restore_seekable_snapshot<R>(
    journal: &DurableJournal,
    reader: &mut R,
) -> Result<SnapshotMetadata>
where
    R: Read + Seek,
{
    if journal.last_cursor() != 0 {
        bail!("snapshot restore requires an empty Sift data directory");
    }

    reader
        .seek(SeekFrom::Start(0))
        .context("seek to the start of the Sift snapshot")?;
    let validated = read_snapshot(reader, |_| Ok(()))?;

    reader
        .seek(SeekFrom::Start(0))
        .context("rewind the validated Sift snapshot")?;
    let mut page = Vec::with_capacity(SNAPSHOT_PAGE_EVENTS);
    let restored = read_snapshot(reader, |event| {
        page.push(event);
        if page.len() == SNAPSHOT_PAGE_EVENTS {
            journal.restore_stored_page(std::mem::take(&mut page))?;
            page.reserve(SNAPSHOT_PAGE_EVENTS);
        }
        Ok(())
    })?;
    if !page.is_empty() {
        journal.restore_stored_page(page)?;
    }
    if restored != validated {
        bail!("Sift snapshot metadata changed between validation and restore");
    }
    if journal.total_event_count() != restored.event_count
        || journal.last_cursor() != restored.last_cursor
    {
        bail!("restored Sift snapshot does not match its declared event count and cursor");
    }
    Ok(restored)
}

/// Spool a non-seekable Raft snapshot in the data directory, validate it, and
/// then restore it in bounded pages. The temporary file stays on the same file
/// system as all other Sift atomic work.
pub(crate) fn restore_streamed_snapshot(
    journal: &DurableJournal,
    reader: &mut dyn Read,
) -> Result<SnapshotMetadata> {
    let tmp_dir = journal.data_dir().join("tmp");
    let mut spool = tempfile::tempfile_in(&tmp_dir)
        .with_context(|| format!("create snapshot spool in {}", tmp_dir.display()))?;
    std::io::copy(reader, &mut spool).context("spool incoming Sift snapshot")?;
    spool
        .sync_all()
        .context("sync incoming Sift snapshot spool")?;
    restore_seekable_snapshot(journal, &mut spool)
}

fn write_snapshot_header(writer: &mut dyn Write, metadata: SnapshotMetadata) -> Result<()> {
    let mut header = Vec::with_capacity(SNAPSHOT_HEADER_BYTES);
    header.extend_from_slice(SNAPSHOT_MAGIC);
    header.extend_from_slice(&SNAPSHOT_FORMAT_VERSION.to_le_bytes());
    header.extend_from_slice(&0_u16.to_le_bytes());
    header.extend_from_slice(&metadata.applied_index.to_le_bytes());
    header.extend_from_slice(&metadata.last_cursor.to_le_bytes());
    header.extend_from_slice(&metadata.event_count.to_le_bytes());
    let checksum = crc32fast::hash(&header);
    header.extend_from_slice(&checksum.to_le_bytes());
    debug_assert_eq!(header.len(), SNAPSHOT_HEADER_BYTES);
    writer
        .write_all(&header)
        .context("write Sift snapshot header")
}

fn write_snapshot_event(writer: &mut dyn Write, stored: &StoredEvent) -> Result<()> {
    let payload = serde_json::to_vec(stored).context("encode Sift snapshot event")?;
    if payload.len() > MAX_SNAPSHOT_EVENT_BYTES {
        bail!(
            "Sift snapshot event at cursor {} exceeds the {} byte limit",
            stored.cursor,
            MAX_SNAPSHOT_EVENT_BYTES
        );
    }
    let payload_len = u32::try_from(payload.len()).context("snapshot event length exceeds u32")?;
    let checksum = crc32fast::hash(&payload);
    writer
        .write_all(&stored.cursor.to_le_bytes())
        .context("write Sift snapshot event cursor")?;
    writer
        .write_all(&payload_len.to_le_bytes())
        .context("write Sift snapshot event length")?;
    writer
        .write_all(&checksum.to_le_bytes())
        .context("write Sift snapshot event checksum")?;
    writer
        .write_all(&payload)
        .context("write Sift snapshot event payload")
}

fn read_snapshot<R, F>(reader: &mut R, mut on_event: F) -> Result<SnapshotMetadata>
where
    R: Read,
    F: FnMut(StoredEvent) -> Result<()>,
{
    let metadata = read_snapshot_header(reader)?;
    let mut expected_cursor = 1_u64;
    let mut last_cursor = 0_u64;
    for position in 0..metadata.event_count {
        let mut frame = [0_u8; SNAPSHOT_FRAME_HEADER_BYTES];
        reader
            .read_exact(&mut frame)
            .with_context(|| format!("read Sift snapshot frame {position}"))?;
        let cursor = u64::from_le_bytes(frame[0..8].try_into().unwrap());
        let payload_len = u32::from_le_bytes(frame[8..12].try_into().unwrap()) as usize;
        let expected_checksum = u32::from_le_bytes(frame[12..16].try_into().unwrap());
        if payload_len == 0 || payload_len > MAX_SNAPSHOT_EVENT_BYTES {
            bail!("Sift snapshot event at cursor {cursor} has invalid length {payload_len}");
        }
        let mut payload = vec![0_u8; payload_len];
        reader
            .read_exact(&mut payload)
            .with_context(|| format!("read Sift snapshot event payload at cursor {cursor}"))?;
        let actual_checksum = crc32fast::hash(&payload);
        if actual_checksum != expected_checksum {
            bail!("Sift snapshot checksum mismatch at cursor {cursor}");
        }
        let stored: StoredEvent = serde_json::from_slice(&payload)
            .with_context(|| format!("decode Sift snapshot event at cursor {cursor}"))?;
        if cursor != stored.cursor {
            bail!(
                "Sift snapshot frame cursor {cursor} does not match payload cursor {}",
                stored.cursor
            );
        }
        if cursor != expected_cursor {
            bail!("Sift snapshot cursor {cursor} is out of order; expected {expected_cursor}");
        }
        stored
            .event
            .validate()
            .with_context(|| format!("validate Sift snapshot event at cursor {cursor}"))?;
        on_event(stored)?;
        last_cursor = cursor;
        expected_cursor = expected_cursor
            .checked_add(1)
            .context("snapshot cursor exhausted u64")?;
    }
    if last_cursor != metadata.last_cursor {
        bail!(
            "Sift snapshot ended at cursor {last_cursor}, expected {}",
            metadata.last_cursor
        );
    }
    if metadata.event_count == 0 && metadata.last_cursor != 0 {
        bail!("empty Sift snapshot declares a non-zero last cursor");
    }
    if read_one_or_eof(reader)?.is_some() {
        bail!("Sift snapshot contains trailing bytes");
    }
    Ok(metadata)
}

fn read_snapshot_header(reader: &mut dyn Read) -> Result<SnapshotMetadata> {
    let mut header = [0_u8; SNAPSHOT_HEADER_BYTES];
    reader
        .read_exact(&mut header)
        .context("read Sift snapshot header")?;
    if &header[..SNAPSHOT_MAGIC.len()] != SNAPSHOT_MAGIC {
        if matches!(header.first(), Some(b'{') | Some(b'[')) {
            bail!("legacy Sift JSON snapshot is unsupported; create a new empty data directory");
        }
        bail!("invalid Sift snapshot magic");
    }
    let version = u16::from_le_bytes(header[8..10].try_into().unwrap());
    if version != SNAPSHOT_FORMAT_VERSION {
        bail!("unsupported Sift snapshot format version {version}");
    }
    let flags = u16::from_le_bytes(header[10..12].try_into().unwrap());
    if flags != 0 {
        bail!("unsupported Sift snapshot flags {flags}");
    }
    let expected_checksum = u32::from_le_bytes(header[36..40].try_into().unwrap());
    let actual_checksum = crc32fast::hash(&header[..36]);
    if actual_checksum != expected_checksum {
        bail!("Sift snapshot header checksum mismatch");
    }
    Ok(SnapshotMetadata {
        applied_index: u64::from_le_bytes(header[12..20].try_into().unwrap()),
        last_cursor: u64::from_le_bytes(header[20..28].try_into().unwrap()),
        event_count: u64::from_le_bytes(header[28..36].try_into().unwrap()),
    })
}

fn read_one_or_eof(reader: &mut dyn Read) -> Result<Option<u8>> {
    let mut byte = [0_u8; 1];
    loop {
        match reader.read(&mut byte) {
            Ok(0) => return Ok(None),
            Ok(_) => return Ok(Some(byte[0])),
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(error).context("check Sift snapshot end"),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ControlState {
    format_version: u16,
    applied_index: u64,
}

impl Default for ControlState {
    fn default() -> Self {
        Self {
            format_version: CONTROL_STATE_FORMAT_VERSION,
            applied_index: 0,
        }
    }
}

/// Applies one committed Raft batch to the canonical per-signal WAL.
pub struct SiftStateMachine {
    journal: Arc<DurableJournal>,
    control_path: PathBuf,
    control: Mutex<ControlState>,
    append_outcomes: Mutex<BTreeMap<u64, Vec<AppendResult>>>,
    applied_index: AtomicU64,
}

impl SiftStateMachine {
    pub fn new(journal: Arc<DurableJournal>) -> Self {
        let data_dir = journal.data_dir().to_path_buf();
        Self::open(data_dir, journal).expect("open Sift state-machine control state")
    }

    pub fn open(data_dir: impl AsRef<Path>, journal: Arc<DurableJournal>) -> Result<Self> {
        let control_dir = data_dir.as_ref().join("control");
        fs::create_dir_all(&control_dir)
            .with_context(|| format!("create Sift control directory {}", control_dir.display()))?;
        set_directory_mode(&control_dir)?;
        let control_path = control_dir.join(CONTROL_STATE_FILE);
        let control =
            if control_path.exists() {
                serde_json::from_slice::<ControlState>(&fs::read(&control_path).with_context(
                    || format!("read Sift control state {}", control_path.display()),
                )?)
                .with_context(|| format!("decode Sift control state {}", control_path.display()))?
            } else {
                ControlState::default()
            };
        if control.format_version != CONTROL_STATE_FORMAT_VERSION {
            bail!(
                "unsupported Sift control state format {}",
                control.format_version
            );
        }
        persist_control(&control_path, &control)?;
        let applied_index = control.applied_index;
        Ok(Self {
            journal,
            control_path,
            control: Mutex::new(control),
            append_outcomes: Mutex::new(BTreeMap::new()),
            applied_index: AtomicU64::new(applied_index),
        })
    }

    pub fn applied_commit_index(&self) -> u64 {
        self.applied_index.load(Ordering::Acquire)
    }

    pub fn apply_local(&self, index: u64, command: &[u8]) -> Result<()> {
        <Self as RaftStateMachine>::apply(self, index, command)
    }

    pub fn take_append_outcomes(&self, index: u64) -> Option<Vec<AppendResult>> {
        self.append_outcomes
            .lock()
            .expect("Sift append outcome lock poisoned")
            .remove(&index)
    }
}

impl RaftStateMachine for SiftStateMachine {
    fn apply(&self, index: Index, command: &[u8]) -> Result<()> {
        if index <= self.applied_index.load(Ordering::Acquire) {
            return Ok(());
        }
        if command.len() > RAFT_BATCH_MAX_BYTES {
            bail!("Sift Raft batch exceeds the 1 MiB limit");
        }
        let command = decode_command(command)?;
        let results = match command {
            SiftCommandV1::AppendEvents { events } => {
                validate_events(&events)?;
                self.journal
                    .append_durable_batch(events)?
                    .into_iter()
                    .map(|result| result.with_commit_index(index))
                    .collect()
            }
        };

        let mut control = self
            .control
            .lock()
            .expect("Sift control state lock poisoned");
        control.applied_index = index;
        persist_control(&self.control_path, &control)?;
        self.append_outcomes
            .lock()
            .expect("Sift append outcome lock poisoned")
            .insert(index, results);
        self.applied_index.store(index, Ordering::Release);
        Ok(())
    }

    fn snapshot(&self, writer: &mut dyn std::io::Write) -> Result<()> {
        let applied_index = self
            .control
            .lock()
            .expect("Sift control state lock poisoned")
            .applied_index;
        write_snapshot(&self.journal, applied_index, writer)?;
        Ok(())
    }

    fn restore(&self, reader: &mut dyn std::io::Read) -> Result<()> {
        let snapshot = restore_streamed_snapshot(&self.journal, reader)?;
        let restored = ControlState {
            format_version: CONTROL_STATE_FORMAT_VERSION,
            applied_index: snapshot.applied_index,
        };
        persist_control(&self.control_path, &restored)?;
        *self
            .control
            .lock()
            .expect("Sift control state lock poisoned") = restored;
        self.applied_index
            .store(snapshot.applied_index, Ordering::Release);
        Ok(())
    }

    fn applied_index(&self) -> Index {
        self.applied_index.load(Ordering::Acquire)
    }
}

fn validate_events(events: &[EventEnvelope]) -> Result<()> {
    if events.is_empty() {
        bail!("Sift Raft batch must not be empty");
    }
    let signal = events[0].signal;
    if events.iter().any(|event| event.signal != signal) {
        bail!("Sift Raft batch must contain exactly one signal");
    }
    for event in events {
        event.validate()?;
    }
    Ok(())
}

fn decode_command(bytes: &[u8]) -> Result<SiftCommandV1> {
    serde_json::from_slice(bytes).context("decode Sift command v1")
}

fn persist_control(path: &Path, control: &ControlState) -> Result<()> {
    storage_durable::atomic_write(
        path,
        &serde_json::to_vec_pretty(control)?,
        storage_durable::FsyncPolicy::Always,
    )
    .with_context(|| format!("atomically persist Sift control state {}", path.display()))?;
    set_file_mode(path)
}

#[cfg(unix)]
fn set_directory_mode(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))
        .with_context(|| format!("set private control directory mode on {}", path.display()))
}

#[cfg(not(unix))]
fn set_directory_mode(_path: &Path) -> Result<()> {
    Ok(())
}

#[cfg(unix)]
fn set_file_mode(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))
        .with_context(|| format!("set private control file mode on {}", path.display()))
}

#[cfg(not(unix))]
fn set_file_mode(_path: &Path) -> Result<()> {
    Ok(())
}

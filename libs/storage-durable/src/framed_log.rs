// CODEGEN-BEGIN
//! Append log whose tail is allowed to be garbage.
//!
//! A frame is a 16-byte header -- `seq: u64 LE`, `len: u32 LE`, `crc32: u32 LE`
//! -- followed by `len` payload bytes. **The CRC covers the payload only, never
//! the header.** A corrupted header cannot announce itself as corrupt: it is
//! read as some plausible `(seq, len)` pair and rejected when `len` exceeds the
//! fixed payload limit, runs past EOF, or points at a payload that fails its
//! CRC. That is the whole torn-tail rule -- the first frame failing any check
//! ends the log, and every byte after it is discarded, unexamined.
//!
//! So reading is not read-only. [`FramedLogWriter::open`] scans for the last
//! good frame end and `set_len`s the file down to it before appending, which
//! means opening a torn log **truncates it**. Recovery happens on open, not on
//! replay.
//!
//! Both sequence arguments are exclusive, in opposite-looking APIs:
//! `read_frames(path, from_seq)` and `replay` keep `seq > from_seq`, and
//! `truncate_through(through)` keeps `seq > through`. `replay` returns the
//! highest seq it actually applied, so replaying a range with nothing in it
//! returns `0` rather than the log's real maximum.
//!
//! `truncate_through` is a full rewrite -- read every frame, write the survivors
//! to `<path>.compact.tmp`, fsync, rename, fsync the parent, re-open in append
//! mode. It is not an in-place hole punch, and its cost is the whole log.
//!
//! The `EverySec` interval is fixed at one second in `open` and is not
//! configurable by any public method.
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};
use memmap2::{Mmap, MmapOptions};

use crate::{strict_sync_parent_dir, sync_parent_dir, FsyncPolicy};

const HEADER_LEN: usize = 16;

/// Largest payload accepted by the shared framed-log format.
///
/// This bound prevents a corrupt or hostile header from forcing a multi-GiB
/// allocation during recovery. Callers must split larger logical records.
pub const MAX_FRAME_PAYLOAD_BYTES: usize = 64 * 1024 * 1024;

/// Test seam for observing frames covered by an AOF trim.
#[doc(hidden)]
pub trait FramedLogTrimObserver: Send + Sync + std::fmt::Debug {
    fn covered_frame(&self, through: u64, seq: u64);

    fn before_temp_sync(&self, _through: u64) {}
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TrimPlanState {
    Prepared,
    Copying,
    Ready,
    Failed,
    Published,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct TrimFileIdentity {
    device: u64,
    inode: u64,
}

/// One staged rewrite of a pinned log inode. The old log stays authoritative
/// while callers release the writer mutex to copy and sync its stable prefix.
/// Finalization includes every later append before replacing the live file.
/// This staged API requires Unix positional reads and stable inode identity.
#[derive(Debug)]
pub struct FramedLogTrimPlan {
    through: u64,
    source: File,
    source_identity: TrimFileIdentity,
    source_end: u64,
    tmp: PathBuf,
    temp: Option<BufWriter<File>>,
    identity: Arc<()>,
    revision: u64,
    id: u64,
    active: Arc<Mutex<Option<u64>>>,
    state: TrimPlanState,
    observer: Option<Arc<dyn FramedLogTrimObserver>>,
    #[cfg(test)]
    faults: Arc<TrimFaults>,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TrimFaultPoint {
    PrefixTempSync,
    SuffixTempSync,
    ParentSyncAfterRename,
}

#[cfg(test)]
#[derive(Debug, Default)]
struct TrimFaults(Mutex<Option<TrimFaultPoint>>);

#[cfg(test)]
impl TrimFaults {
    fn fail_once(&self, point: TrimFaultPoint) {
        *self.0.lock().unwrap() = Some(point);
    }

    fn check(&self, point: TrimFaultPoint) -> Result<()> {
        let mut next = self.0.lock().unwrap();
        if *next == Some(point) {
            *next = None;
            bail!("injected trim I/O failure: {point:?}");
        }
        Ok(())
    }
}

/// One validated log frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogFrame {
    pub seq: u64,
    pub payload: Vec<u8>,
}

/// A validated read-only frame view.
///
/// The completed implementation owns its file mapping, so this view remains
/// valid after its cursor is dropped or the original path is replaced.
#[derive(Debug)]
pub struct MappedLogFrame {
    pub seq: u64,
    payload: Option<Mmap>,
}

impl MappedLogFrame {
    pub fn payload(&self) -> &[u8] {
        self.payload.as_deref().unwrap_or_default()
    }
}

/// CRC-framed append log with clean torn-tail recovery.
pub struct FramedLogWriter {
    path: PathBuf,
    file: BufWriter<File>,
    policy: FsyncPolicy,
    last_sync: Instant,
    sync_every: Duration,
    dirty: bool,
    trim_observer: Option<Arc<dyn FramedLogTrimObserver>>,
    trim_identity: Arc<()>,
    trim_revision: u64,
    trim_active: Arc<Mutex<Option<u64>>>,
    trim_next_id: u64,
    #[cfg(test)]
    trim_faults: Arc<TrimFaults>,
}

impl FramedLogWriter {
    pub fn open(path: impl Into<PathBuf>, policy: FsyncPolicy) -> Result<Self> {
        let path = path.into();
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)
                    .with_context(|| format!("create log dir {}", parent.display()))?;
            }
        }

        let good_end = if path.exists() {
            FramedLogReader::scan_good_end(&path)?
        } else {
            0
        };
        if path.exists() {
            let file = OpenOptions::new()
                .write(true)
                .open(&path)
                .with_context(|| format!("open log for truncate {}", path.display()))?;
            file.set_len(good_end)
                .with_context(|| format!("truncate log tail {}", path.display()))?;
            if policy != FsyncPolicy::Os {
                file.sync_all()
                    .with_context(|| format!("fsync truncated log {}", path.display()))?;
                sync_parent_dir(&path)?;
            }
        }

        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .append(true)
            .open(&path)
            .with_context(|| format!("open log for append {}", path.display()))?;
        Ok(Self {
            path,
            file: BufWriter::new(file),
            policy,
            last_sync: Instant::now(),
            sync_every: Duration::from_secs(1),
            dirty: false,
            trim_observer: None,
            trim_identity: Arc::new(()),
            trim_revision: 0,
            trim_active: Arc::new(Mutex::new(None)),
            trim_next_id: 1,
            #[cfg(test)]
            trim_faults: Arc::new(TrimFaults::default()),
        })
    }

    #[doc(hidden)]
    pub fn with_trim_observer(mut self, observer: Arc<dyn FramedLogTrimObserver>) -> Self {
        self.trim_observer = Some(observer);
        self
    }

    pub fn append(&mut self, seq: u64, payload: &[u8]) -> Result<()> {
        let len = checked_payload_len(payload.len())?;
        self.append_validated(seq, len, payload)
    }

    /// Append one format-valid frame without imposing the owned-frame limit.
    ///
    /// This is for callers that retain the payload in borrowed storage, such
    /// as an mmap. It still refuses lengths that the u32 frame format cannot
    /// represent.
    pub fn append_large_payload(&mut self, seq: u64, payload: &[u8]) -> Result<()> {
        let len = checked_legacy_payload_len(payload.len())?;
        self.append_validated(seq, len, payload)
    }

    fn append_validated(&mut self, seq: u64, len: u32, payload: &[u8]) -> Result<()> {
        let crc = crc32fast::hash(payload);
        let mut header = [0u8; HEADER_LEN];
        header[0..8].copy_from_slice(&seq.to_le_bytes());
        header[8..12].copy_from_slice(&len.to_le_bytes());
        header[12..16].copy_from_slice(&crc.to_le_bytes());
        self.file.write_all(&header).context("write log header")?;
        self.file.write_all(payload).context("write log payload")?;
        self.dirty = true;
        if self.policy.should_sync_immediately() {
            self.sync()?;
        }
        Ok(())
    }

    pub fn flush(&mut self) -> Result<()> {
        self.file.flush().context("flush log writer")
    }

    pub fn sync(&mut self) -> Result<()> {
        self.flush()?;
        self.file.get_ref().sync_all().context("fsync log")?;
        sync_parent_dir(&self.path)?;
        self.last_sync = Instant::now();
        self.dirty = false;
        Ok(())
    }

    /// Flush and fsync the log, then require a successful parent-directory
    /// fsync. This is the pre-commit boundary used by durable replacement.
    pub fn sync_strict(&mut self) -> Result<()> {
        self.flush()?;
        self.file.get_ref().sync_all().context("strict fsync log")?;
        strict_sync_parent_dir(&self.path)?;
        self.last_sync = Instant::now();
        self.dirty = false;
        Ok(())
    }

    pub fn maybe_sync(&mut self) -> Result<()> {
        if self.policy != FsyncPolicy::EverySec {
            return Ok(());
        }
        if self.dirty && self.last_sync.elapsed() >= self.sync_every {
            self.sync()?;
        }
        Ok(())
    }

    pub fn truncate_through(&mut self, through: u64) -> Result<()> {
        let active = Arc::clone(&self.trim_active);
        let owner = active.lock().map_err(|_| anyhow::anyhow!("trim ownership poisoned"))?;
        if owner.is_some() {
            bail!("trim plan Busy");
        }
        let revision = self.trim_revision.checked_add(1).context("trim revision exhausted")?;
        self.flush()?;
        let mut frames = FramedLogCursor::open(&self.path)?;
        let tmp = self.compact_tmp_path();
        remove_abandoned_trim_temp(&tmp)?;
        let mut dst = BufWriter::new(
            OpenOptions::new().create_new(true).read(true).append(true).open(&tmp)
                .with_context(|| format!("create log compaction temp {}", tmp.display()))?,
        );
        // Preserve the existing owned-frame payload limit on this separate API.
        while let Some(frame) = frames.next_frame()? {
            if frame.seq > through {
                write_frame(&mut dst, frame.seq, &frame.payload)?;
            }
        }
        dst.flush().context("flush log compaction temp")?;
        dst.get_ref().sync_all().context("fsync log compaction temp")?;
        let file = dst.into_inner().map_err(|error| error.into_error())
            .context("retain log compaction temp handle")?;
        std::fs::rename(&tmp, &self.path).context("commit log compaction")?;
        // Keep the published handle even if the directory sync fails.
        self.file = BufWriter::new(file);
        self.trim_revision = revision;
        sync_parent_dir(&self.path)?;
        self.dirty = false;
        self.last_sync = Instant::now();
        Ok(())
    }

    /// Capture a complete source boundary while the caller owns the writer.
    /// Flush buffers here; scanning and syncing belong to the unlocked plan.
    pub fn begin_trim_mapped(&mut self, through: u64) -> Result<FramedLogTrimPlan> {
        let active = Arc::clone(&self.trim_active);
        let mut owner = active.lock().map_err(|_| anyhow::anyhow!("trim ownership poisoned"))?;
        if owner.is_some() {
            bail!("trim plan Busy");
        }
        let id = self.trim_next_id;
        let next_id = id.checked_add(1).context("trim identity exhausted")?;
        self.flush()?;
        let source = self.file.get_ref().try_clone().context("pin live trim source")?;
        let metadata = source.metadata()?;
        let source_identity = trim_file_identity(&metadata)?;
        if trim_file_identity(&std::fs::symlink_metadata(&self.path)?)? != source_identity {
            bail!("trim source path changed");
        }
        let tmp = self.compact_tmp_path();
        remove_abandoned_trim_temp(&tmp)?;
        let temp = BufWriter::new(
            OpenOptions::new().create_new(true).read(true).append(true).open(&tmp)
                .with_context(|| format!("create log compaction temp {}", tmp.display()))?,
        );
        // Nothing fallible follows ownership publication. A failed open
        // cannot strand a phantom Busy plan.
        *owner = Some(id);
        self.trim_next_id = next_id;
        Ok(FramedLogTrimPlan {
            through,
            source,
            source_identity,
            source_end: metadata.len(),
            tmp,
            temp: Some(temp),
            identity: Arc::clone(&self.trim_identity),
            revision: self.trim_revision,
            id,
            active: Arc::clone(&active),
            state: TrimPlanState::Prepared,
            observer: self.trim_observer.clone(),
            #[cfg(test)]
            faults: Arc::clone(&self.trim_faults),
        })
    }

    /// Include the suffix appended during the unlocked copy, then publish.
    /// The caller owns the writer again. Suffix and directory sync can still
    /// wait for the filesystem; this does not impose a time bound.
    pub fn finish_trim_mapped(&mut self, mut plan: FramedLogTrimPlan) -> Result<()> {
        if plan.state != TrimPlanState::Ready
            || !Arc::ptr_eq(&plan.identity, &self.trim_identity)
            || plan.revision != self.trim_revision
            || *self.trim_active.lock().map_err(|_| anyhow::anyhow!("trim ownership poisoned"))?
                != Some(plan.id)
        {
            bail!("trim plan is not publishable");
        }
        let revision = self.trim_revision.checked_add(1).context("trim revision exhausted")?;
        if trim_file_identity(&self.file.get_ref().metadata()?)? != plan.source_identity
            || trim_file_identity(&plan.source.metadata()?)? != plan.source_identity
            || trim_file_identity(&std::fs::symlink_metadata(&self.path)?)? != plan.source_identity
        {
            bail!("trim source path changed");
        }
        self.flush()?;
        let current_end = self.file.get_ref().metadata()?.len();
        if current_end < plan.source_end {
            bail!("trim source shrank below captured boundary");
        }
        let temp = plan.temp.as_mut().context("missing trim temp")?;
        let added = stream_trim_range(
            &plan.source, plan.source_end, current_end, plan.through, temp, None,
        )?;
        temp.flush().context("flush log compaction suffix")?;
        if added {
            #[cfg(test)]
            plan.faults.check(TrimFaultPoint::SuffixTempSync)?;
            temp.get_ref().sync_all().context("fsync log compaction suffix")?;
        }
        if trim_file_identity(&temp.get_ref().metadata()?)?
            != trim_file_identity(&std::fs::symlink_metadata(&plan.tmp)?)?
        {
            bail!("trim temporary path changed");
        }
        let file = plan.temp.take().context("missing trim temp")?.into_inner()
            .map_err(|error| error.into_error()).context("retain log compaction temp handle")?;
        std::fs::rename(&plan.tmp, &self.path).context("commit log compaction")?;
        self.file = BufWriter::new(file);
        self.trim_revision = revision;
        plan.state = TrimPlanState::Published;
        #[cfg(test)]
        plan.faults.check(TrimFaultPoint::ParentSyncAfterRename)?;
        sync_parent_dir(&self.path)?;
        self.dirty = false;
        self.last_sync = Instant::now();
        // Drop clears only this plan's ownership, also after a sync error.
        Ok(())
    }

    /// Rewrite complete CRC-valid u32-sized frames through the cut.
    /// Unix callers may split begin/copy/finish to release an external mutex.
    /// Other platforms retain the existing one-phase mapped rewrite.
    pub fn truncate_through_mapped(&mut self, through: u64) -> Result<()> {
        #[cfg(unix)]
        {
            let mut plan = self.begin_trim_mapped(through)?;
            plan.copy_stable_prefix()?;
            self.finish_trim_mapped(plan)
        }
        #[cfg(not(unix))]
        {
            self.truncate_mapped_in_place(through)
        }
    }

    // Keep the established synchronous API on platforms without Unix inode
    // identity. Tests also exercise this implementation on the build host.
    #[cfg(any(not(unix), test))]
    fn truncate_mapped_in_place(&mut self, through: u64) -> Result<()> {
        let active = Arc::clone(&self.trim_active);
        let owner = active.lock().map_err(|_| anyhow::anyhow!("trim ownership poisoned"))?;
        if owner.is_some() {
            bail!("trim plan Busy");
        }
        let revision = self.trim_revision.checked_add(1).context("trim revision exhausted")?;
        self.flush()?;
        let mut frames = FramedLogCursor::open(&self.path)?;
        let tmp = self.compact_tmp_path();
        remove_abandoned_trim_temp(&tmp)?;
        let file = {
            let mut dst = BufWriter::new(
                OpenOptions::new()
                    .create_new(true)
                    .read(true)
                    .append(true)
                    .open(&tmp)
                    .with_context(|| format!("create log compaction temp {}", tmp.display()))?,
            );
            while let Some(frame) = frames.next_large_mapped_frame()? {
                if frame.seq <= through {
                    if let Some(observer) = &self.trim_observer {
                        observer.covered_frame(through, frame.seq);
                    }
                    continue;
                }
                write_large_frame(&mut dst, frame.seq, frame.payload())?;
            }
            dst.flush().context("flush log compaction temp")?;
            if let Some(observer) = &self.trim_observer {
                observer.before_temp_sync(through);
            }
            dst.get_ref()
                .sync_all()
                .context("fsync log compaction temp")?;
            dst.into_inner()
                .map_err(|error| error.into_error())
                .context("retain log compaction temp handle")?
        };
        std::fs::rename(&tmp, &self.path).with_context(|| {
            format!(
                "commit log compaction {} -> {}",
                tmp.display(),
                self.path.display()
            )
        })?;
        // Install the staged inode before the directory fsync. If that fsync
        // reports an error, later appends still target the published log.
        self.file = BufWriter::new(file);
        self.trim_revision = revision;
        sync_parent_dir(&self.path)?;
        self.dirty = false;
        self.last_sync = Instant::now();
        Ok(())
    }

    #[cfg(test)]
    fn trim_faults_for_test(&self) -> Arc<TrimFaults> {
        Arc::clone(&self.trim_faults)
    }

    fn compact_tmp_path(&self) -> PathBuf {
        let mut tmp = self.path.as_os_str().to_os_string();
        tmp.push(".compact.tmp");
        tmp.into()
    }
}

impl FramedLogTrimPlan {
    /// Validate and copy the whole captured prefix, then durably sync it.
    /// This has no writer borrow and never changes the live log.
    pub fn copy_stable_prefix(&mut self) -> Result<()> {
        if self.state != TrimPlanState::Prepared {
            self.state = TrimPlanState::Failed;
            bail!("trim plan is not prepared");
        }
        self.state = TrimPlanState::Copying;
        let result = (|| {
            let temp = self.temp.as_mut().context("missing trim temp")?;
            stream_trim_range(
                &self.source, 0, self.source_end, self.through, temp, self.observer.as_deref(),
            )?;
            temp.flush().context("flush log compaction temp")?;
            if let Some(observer) = &self.observer {
                observer.before_temp_sync(self.through);
            }
            #[cfg(test)]
            self.faults.check(TrimFaultPoint::PrefixTempSync)?;
            temp.get_ref().sync_all().context("fsync log compaction temp")?;
            Ok(())
        })();
        self.state = if result.is_ok() { TrimPlanState::Ready } else { TrimPlanState::Failed };
        result
    }
}

impl Drop for FramedLogTrimPlan {
    fn drop(&mut self) {
        let mut owner = self.active.lock().unwrap_or_else(|error| error.into_inner());
        if *owner == Some(self.id) {
            // Keep ownership until removal completes, so the next begin
            // cannot create its temp between clearing this ID and unlinking.
            if let Some(temp) = self.temp.take() {
                // Discard unpublished buffered bytes without issuing another
                // write from Drop after an earlier I/O failure.
                drop(temp.into_parts());
            }
            if self.state != TrimPlanState::Published {
                let _ = std::fs::remove_file(&self.tmp);
            }
            *owner = None;
        }
    }
}

fn remove_abandoned_trim_temp(path: &Path) -> Result<()> {
    match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error).context("remove abandoned log compaction temp"),
    }
}

fn trim_file_identity(metadata: &std::fs::Metadata) -> Result<TrimFileIdentity> {
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        bail!("trim requires a regular nonsymlink file");
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        Ok(TrimFileIdentity { device: metadata.dev(), inode: metadata.ino() })
    }
    #[cfg(not(unix))]
    bail!("staged log trim requires Unix inode identity")
}

fn trim_read_exact_at(file: &File, mut bytes: &mut [u8], mut offset: u64) -> Result<()> {
    while !bytes.is_empty() {
        #[cfg(unix)]
        let read = {
            use std::os::unix::fs::FileExt;
            file.read_at(bytes, offset)
        };
        #[cfg(not(unix))]
        let read: std::io::Result<usize> = Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported, "staged log trim requires positional reads",
        ));
        match read {
            Ok(0) => bail!("trim source ended before captured EOF"),
            Ok(count) => {
                offset += count as u64;
                bytes = &mut bytes[count..];
            }
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(error).context("read pinned trim source"),
        }
    }
    Ok(())
}

/// Read a private, writer-captured frame interval. Unlike torn-tail recovery,
/// trim refuses every incomplete frame. The temp remains unpublished while
/// streaming, so a CRC failure discards the plan, including partial output.
fn stream_trim_range(
    source: &File,
    mut offset: u64,
    end: u64,
    through: u64,
    temp: &mut BufWriter<File>,
    observer: Option<&dyn FramedLogTrimObserver>,
) -> Result<bool> {
    if offset > end {
        bail!("trim frame interval is reversed");
    }
    let mut buffer = vec![0u8; 64 * 1024];
    let mut added = false;
    while offset < end {
        let frame_start = offset;
        if end - offset < HEADER_LEN as u64 {
            bail!("trim source contains a torn frame header");
        }
        let mut header = [0u8; HEADER_LEN];
        trim_read_exact_at(source, &mut header, offset)?;
        let seq = u64::from_le_bytes(header[0..8].try_into().unwrap());
        let len = u32::from_le_bytes(header[8..12].try_into().unwrap()) as u64;
        let expected_crc = u32::from_le_bytes(header[12..16].try_into().unwrap());
        offset += HEADER_LEN as u64;
        if len > end - offset {
            if len > MAX_FRAME_PAYLOAD_BYTES as u64 {
                bail!("oversized legacy log frame at byte {frame_start} is incomplete; refusing destructive recovery");
            }
            bail!("trim source contains a torn frame payload");
        }
        let keep = seq > through;
        if keep {
            temp.write_all(&header).context("write trim frame header")?;
            added = true;
        }
        let mut crc = crc32fast::Hasher::new();
        let mut remaining = len;
        while remaining != 0 {
            let count = remaining.min(buffer.len() as u64) as usize;
            trim_read_exact_at(source, &mut buffer[..count], offset)?;
            crc.update(&buffer[..count]);
            if keep {
                temp.write_all(&buffer[..count]).context("write trim frame payload")?;
            }
            offset += count as u64;
            remaining -= count as u64;
        }
        if crc.finalize() != expected_crc {
            if len > MAX_FRAME_PAYLOAD_BYTES as u64 {
                bail!("oversized legacy log frame at byte {frame_start} failed validation; refusing destructive recovery");
            }
            bail!("trim source frame CRC mismatch");
        }
        if !keep {
            if let Some(observer) = observer {
                observer.covered_frame(through, seq);
            }
        }
    }
    Ok(added)
}

/// Stateful reader for one validated frame at a time.
///
/// The cursor keeps its byte offset. Repeated calls do not scan the skipped
/// prefix again, and memory is bounded by the current frame payload.
pub struct FramedLogCursor {
    file: Option<File>,
    total: u64,
    offset: u64,
    header: [u8; HEADER_LEN],
}

impl FramedLogCursor {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        if !path.exists() {
            return Ok(Self {
                file: None,
                total: 0,
                offset: 0,
                header: [0; HEADER_LEN],
            });
        }
        let file = File::open(path).with_context(|| format!("open log {}", path.display()))?;
        let total = file.metadata()?.len();
        Ok(Self {
            file: Some(file),
            total,
            offset: 0,
            header: [0; HEADER_LEN],
        })
    }

    pub fn next_frame(&mut self) -> Result<Option<LogFrame>> {
        let Some(file) = self.file.as_mut() else {
            return Ok(None);
        };
        let Some((seq, payload, next)) =
            read_one_frame(file, self.total, self.offset, &mut self.header)?
        else {
            return Ok(None);
        };
        self.offset = next;
        Ok(Some(LogFrame { seq, payload }))
    }

    /// Return the next validated frame without creating an owned payload Vec.
    pub fn next_mapped_frame(&mut self) -> Result<Option<MappedLogFrame>> {
        let Some(file) = self.file.as_mut() else {
            return Ok(None);
        };
        let Some((frame, next)) =
            read_one_mapped_frame(file, self.total, self.offset, &mut self.header)?
        else {
            return Ok(None);
        };
        self.offset = next;
        Ok(Some(frame))
    }

    /// Return the next CRC-valid frame as a borrowed mapping, including
    /// legacy payloads above [`MAX_FRAME_PAYLOAD_BYTES`].
    pub fn next_large_mapped_frame(&mut self) -> Result<Option<MappedLogFrame>> {
        let Some(file) = self.file.as_mut() else {
            return Ok(None);
        };
        let Some((frame, next)) =
            read_one_large_mapped_frame(file, self.total, self.offset, &mut self.header)?
        else {
            return Ok(None);
        };
        self.offset = next;
        Ok(Some(frame))
    }

    /// Read a validated frame through this cursor's pinned file and length.
    /// The logical offset is unchanged, including when the path was replaced.
    pub fn reread_frame_at(&mut self, offset: u64) -> Result<Option<LogFrame>> {
        if offset
            .checked_add(HEADER_LEN as u64)
            .is_none_or(|end| end > self.total)
        {
            return Ok(None);
        }
        let Some(file) = self.file.as_mut() else {
            return Ok(None);
        };
        read_one_frame(file, self.total, offset, &mut self.header)
            .map(|frame| frame.map(|(seq, payload, _)| LogFrame { seq, payload }))
    }

    /// Reread one mapped frame without changing the replay offset.
    pub fn reread_mapped_frame_at(&mut self, offset: u64) -> Result<Option<MappedLogFrame>> {
        if offset
            .checked_add(HEADER_LEN as u64)
            .is_none_or(|end| end > self.total)
        {
            return Ok(None);
        }
        let Some(file) = self.file.as_mut() else {
            return Ok(None);
        };
        read_one_mapped_frame(file, self.total, offset, &mut self.header)
            .map(|frame| frame.map(|(frame, _)| frame))
    }

    /// Reread a large mapped frame from this cursor's pinned inode and
    /// open-time length without changing the replay offset.
    pub fn reread_large_mapped_frame_at(&mut self, offset: u64) -> Result<Option<MappedLogFrame>> {
        if offset
            .checked_add(HEADER_LEN as u64)
            .is_none_or(|end| end > self.total)
        {
            return Ok(None);
        }
        let Some(file) = self.file.as_mut() else {
            return Ok(None);
        };
        read_one_large_mapped_frame(file, self.total, offset, &mut self.header)
            .map(|frame| frame.map(|(frame, _)| frame))
    }

    pub fn byte_offset(&self) -> u64 {
        self.offset
    }
}

/// Reader for CRC-framed append logs.
pub struct FramedLogReader;

impl FramedLogReader {
    /// Visit validated frames without retaining the complete log in memory.
    ///
    /// The visitor runs in file order. A torn tail ends iteration at the last
    /// complete frame, matching `read_frames` recovery semantics.
    pub fn visit_frames(
        path: impl AsRef<Path>,
        from_seq: u64,
        mut visit: impl FnMut(LogFrame) -> Result<()>,
    ) -> Result<u64> {
        let path = path.as_ref();
        if !path.exists() {
            return Ok(0);
        }
        let mut file = File::open(path).with_context(|| format!("open log {}", path.display()))?;
        let total = file.metadata()?.len();
        let mut off = 0u64;
        let mut max_seq = 0u64;
        let mut header = [0u8; HEADER_LEN];
        loop {
            let Some((seq, payload, next)) = read_one_frame(&mut file, total, off, &mut header)?
            else {
                break;
            };
            if seq > from_seq {
                max_seq = max_seq.max(seq);
                visit(LogFrame { seq, payload })?;
            }
            off = next;
        }
        Ok(max_seq)
    }

    pub fn replay(
        path: impl AsRef<Path>,
        from_seq: u64,
        mut apply: impl FnMut(LogFrame),
    ) -> Result<u64> {
        Self::visit_frames(path, from_seq, |frame| {
            apply(frame);
            Ok(())
        })
    }

    pub fn read_frames(path: impl AsRef<Path>, from_seq: u64) -> Result<Vec<LogFrame>> {
        let path = path.as_ref();
        if !path.exists() {
            return Ok(Vec::new());
        }
        let mut file = File::open(path).with_context(|| format!("open log {}", path.display()))?;
        let total = file.metadata()?.len();
        let mut off = 0u64;
        let mut out = Vec::new();
        let mut header = [0u8; HEADER_LEN];
        loop {
            let Some((seq, payload, next)) = read_one_frame(&mut file, total, off, &mut header)?
            else {
                break;
            };
            if seq > from_seq {
                out.push(LogFrame { seq, payload });
            }
            off = next;
        }
        Ok(out)
    }

    /// Read at most `limit` frames after `from_seq` without retaining the
    /// skipped payloads. Callers that repeatedly page a validated open log use
    /// this to keep recovery memory bounded by one page.
    pub fn read_frames_bounded(
        path: impl AsRef<Path>,
        from_seq: u64,
        limit: usize,
    ) -> Result<Vec<LogFrame>> {
        if limit == 0 {
            return Ok(Vec::new());
        }
        let path = path.as_ref();
        if !path.exists() {
            return Ok(Vec::new());
        }
        let mut file = File::open(path).with_context(|| format!("open log {}", path.display()))?;
        let total = file.metadata()?.len();
        let mut off = 0u64;
        let mut out = Vec::with_capacity(limit.min(1_000));
        let mut header = [0u8; HEADER_LEN];
        while off + HEADER_LEN as u64 <= total && out.len() < limit {
            file.seek(SeekFrom::Start(off))?;
            if file.read_exact(&mut header).is_err() {
                break;
            }
            let seq = u64::from_le_bytes(header[0..8].try_into().expect("fixed sequence bytes"));
            let len =
                u32::from_le_bytes(header[8..12].try_into().expect("fixed length bytes")) as u64;
            let crc = u32::from_le_bytes(header[12..16].try_into().expect("fixed crc bytes"));
            let Some(frame_end) = frame_end(off, len, total) else {
                break;
            };
            if seq > from_seq {
                let mut payload = vec![0u8; len as usize];
                if file.read_exact(&mut payload).is_err() || crc32fast::hash(&payload) != crc {
                    break;
                }
                out.push(LogFrame { seq, payload });
            }
            off = frame_end;
        }
        Ok(out)
    }

    pub fn scan_good_end(path: impl AsRef<Path>) -> Result<u64> {
        let path = path.as_ref();
        let mut file = File::open(path).with_context(|| format!("open log {}", path.display()))?;
        let total = file.metadata()?.len();
        let mut off = 0u64;
        let mut header = [0u8; HEADER_LEN];
        while let Some(next) = scan_one_frame(&mut file, total, off, &mut header)? {
            off = next;
        }
        Ok(off)
    }
}

fn write_frame(mut writer: impl Write, seq: u64, payload: &[u8]) -> Result<()> {
    let len = checked_payload_len(payload.len())?;
    write_frame_with_len(&mut writer, seq, len, payload)
}

#[cfg(any(not(unix), test))]
fn write_large_frame(mut writer: impl Write, seq: u64, payload: &[u8]) -> Result<()> {
    let len = checked_legacy_payload_len(payload.len())?;
    write_frame_with_len(&mut writer, seq, len, payload)
}

fn write_frame_with_len(mut writer: impl Write, seq: u64, len: u32, payload: &[u8]) -> Result<()> {
    let crc = crc32fast::hash(payload);
    let mut header = [0u8; HEADER_LEN];
    header[0..8].copy_from_slice(&seq.to_le_bytes());
    header[8..12].copy_from_slice(&len.to_le_bytes());
    header[12..16].copy_from_slice(&crc.to_le_bytes());
    writer.write_all(&header).context("write log header")?;
    writer.write_all(payload).context("write log payload")?;
    Ok(())
}

fn read_one_frame(
    file: &mut File,
    total: u64,
    off: u64,
    header: &mut [u8; HEADER_LEN],
) -> Result<Option<(u64, Vec<u8>, u64)>> {
    if off + HEADER_LEN as u64 > total {
        return Ok(None);
    }
    file.seek(SeekFrom::Start(off))?;
    if file.read_exact(header).is_err() {
        return Ok(None);
    }
    let seq = u64::from_le_bytes([
        header[0], header[1], header[2], header[3], header[4], header[5], header[6], header[7],
    ]);
    let len = u32::from_le_bytes([header[8], header[9], header[10], header[11]]) as u64;
    let crc = u32::from_le_bytes([header[12], header[13], header[14], header[15]]);
    let Some(frame_end) = complete_frame_end(off, len, total) else {
        if len > MAX_FRAME_PAYLOAD_BYTES as u64 {
            bail!(
                "oversized legacy log frame at byte {off} is incomplete; refusing destructive recovery"
            );
        }
        return Ok(None);
    };
    if len > MAX_FRAME_PAYLOAD_BYTES as u64 {
        validate_payload_crc_streaming(file, len, crc)
            .with_context(|| format!("validate oversized legacy log frame at byte {off}"))?;
        bail!(
            "validated legacy log frame at byte {off} has {len} payload bytes, above the supported read limit {MAX_FRAME_PAYLOAD_BYTES}; the file was not modified"
        );
    }
    let mut payload = vec![0u8; len as usize];
    if file.read_exact(&mut payload).is_err() {
        return Ok(None);
    }
    if crc32fast::hash(&payload) != crc {
        return Ok(None);
    }
    Ok(Some((seq, payload, frame_end)))
}

/// Read and validate one frame without allocating an owned payload buffer.
///
/// The mapping refers to the cursor's already-open file descriptor. Supported
/// writers either append beyond the cursor's pinned `total` length or atomically
/// replace the path with a different inode; neither operation truncates or
/// changes the mapped range of this original inode while a view is alive.
fn read_one_mapped_frame(
    file: &mut File,
    total: u64,
    off: u64,
    header: &mut [u8; HEADER_LEN],
) -> Result<Option<(MappedLogFrame, u64)>> {
    read_one_mapped_frame_with_limit(file, total, off, header, false)
}

fn read_one_large_mapped_frame(
    file: &mut File,
    total: u64,
    off: u64,
    header: &mut [u8; HEADER_LEN],
) -> Result<Option<(MappedLogFrame, u64)>> {
    read_one_mapped_frame_with_limit(file, total, off, header, true)
}

fn read_one_mapped_frame_with_limit(
    file: &mut File,
    total: u64,
    off: u64,
    header: &mut [u8; HEADER_LEN],
    accept_large: bool,
) -> Result<Option<(MappedLogFrame, u64)>> {
    if off
        .checked_add(HEADER_LEN as u64)
        .is_none_or(|end| end > total)
    {
        return Ok(None);
    }
    file.seek(SeekFrom::Start(off))?;
    if file.read_exact(header).is_err() {
        return Ok(None);
    }
    let seq = u64::from_le_bytes([
        header[0], header[1], header[2], header[3], header[4], header[5], header[6], header[7],
    ]);
    let len = u32::from_le_bytes([header[8], header[9], header[10], header[11]]) as u64;
    let crc = u32::from_le_bytes([header[12], header[13], header[14], header[15]]);
    let Some(frame_end) = complete_frame_end(off, len, total) else {
        if len > MAX_FRAME_PAYLOAD_BYTES as u64 {
            bail!(
                "oversized legacy log frame at byte {off} is incomplete; refusing destructive recovery"
            );
        }
        return Ok(None);
    };
    if len > MAX_FRAME_PAYLOAD_BYTES as u64 && !accept_large {
        validate_payload_crc_streaming(file, len, crc)
            .with_context(|| format!("validate oversized legacy log frame at byte {off}"))?;
        bail!(
            "validated legacy log frame at byte {off} has {len} payload bytes, above the supported read limit {MAX_FRAME_PAYLOAD_BYTES}; the file was not modified"
        );
    }
    if let Err(error) = validate_payload_crc_streaming(file, len, crc) {
        if len > MAX_FRAME_PAYLOAD_BYTES as u64 {
            return Err(error).with_context(|| {
                format!("oversized legacy log frame at byte {off} failed validation; refusing destructive recovery")
            });
        }
        return Ok(None);
    }
    let payload = if len == 0 {
        None
    } else {
        let payload_offset = off
            .checked_add(HEADER_LEN as u64)
            .expect("validated frame payload offset fits u64");
        let payload_len = usize::try_from(len).expect("validated frame payload length fits usize");
        // SAFETY: `payload_offset..frame_end` is within the cursor's immutable
        // open-time length and its CRC has just been checked. The supported
        // writer operations append after that length or atomically replace the
        // path, which leaves this pinned inode and mapped byte range intact.
        Some(unsafe {
            MmapOptions::new()
                .offset(payload_offset)
                .len(payload_len)
                .map(&*file)
                .with_context(|| format!("map validated log frame at byte {off}"))?
        })
    };
    Ok(Some((MappedLogFrame { seq, payload }, frame_end)))
}

fn checked_payload_len(len: usize) -> Result<u32> {
    if len > MAX_FRAME_PAYLOAD_BYTES {
        anyhow::bail!("log payload {len} bytes exceeds maximum {MAX_FRAME_PAYLOAD_BYTES} bytes");
    }
    u32::try_from(len).context("log payload too large for u32 len")
}

fn checked_legacy_payload_len(len: usize) -> Result<u32> {
    u32::try_from(len).context("log payload too large for u32 len")
}

fn frame_end(off: u64, len: u64, total: u64) -> Option<u64> {
    if len > MAX_FRAME_PAYLOAD_BYTES as u64 {
        return None;
    }
    complete_frame_end(off, len, total)
}

fn complete_frame_end(off: u64, len: u64, total: u64) -> Option<u64> {
    let end = off.checked_add(HEADER_LEN as u64)?.checked_add(len)?;
    (end <= total).then_some(end)
}

/// Scan one frame without allocating its payload.
///
/// New writers cap frames at [`MAX_FRAME_PAYLOAD_BYTES`]. Older releases
/// accepted every `u32` length. Recovery therefore streams the CRC for a
/// complete legacy frame. An incomplete or corrupt oversized frame is an
/// error, not a torn tail, because truncating it could delete valid data from
/// an older writer.
fn scan_one_frame(
    file: &mut File,
    total: u64,
    off: u64,
    header: &mut [u8; HEADER_LEN],
) -> Result<Option<u64>> {
    if off + HEADER_LEN as u64 > total {
        return Ok(None);
    }
    file.seek(SeekFrom::Start(off))?;
    if file.read_exact(header).is_err() {
        return Ok(None);
    }
    let len = u32::from_le_bytes([header[8], header[9], header[10], header[11]]) as u64;
    let crc = u32::from_le_bytes([header[12], header[13], header[14], header[15]]);
    let Some(end) = complete_frame_end(off, len, total) else {
        if len > MAX_FRAME_PAYLOAD_BYTES as u64 {
            bail!(
                "oversized legacy log frame at byte {off} is incomplete; refusing destructive recovery"
            );
        }
        return Ok(None);
    };
    match validate_payload_crc_streaming(file, len, crc) {
        Ok(()) => Ok(Some(end)),
        Err(error) if len > MAX_FRAME_PAYLOAD_BYTES as u64 => Err(error).with_context(|| {
            format!(
                "oversized legacy log frame at byte {off} failed validation; refusing destructive recovery"
            )
        }),
        Err(_) => Ok(None),
    }
}

fn validate_payload_crc_streaming(file: &mut File, len: u64, expected: u32) -> Result<()> {
    let mut remaining = len;
    let mut buffer = [0u8; 64 * 1024];
    let mut hasher = crc32fast::Hasher::new();
    while remaining > 0 {
        let read_len = usize::try_from(remaining.min(buffer.len() as u64))
            .expect("CRC buffer length fits usize");
        file.read_exact(&mut buffer[..read_len])
            .context("read framed-log payload for streaming CRC")?;
        hasher.update(&buffer[..read_len]);
        remaining -= read_len as u64;
    }
    let actual = hasher.finalize();
    if actual != expected {
        bail!("framed-log payload CRC mismatch: expected {expected}, got {actual}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::alloc::{GlobalAlloc, Layout, System};
    use std::cell::Cell;

    struct FrameAllocationObserver;

    #[global_allocator]
    static FRAME_ALLOCATION_OBSERVER: FrameAllocationObserver = FrameAllocationObserver;

    thread_local! {
        static OBSERVE_ALLOCATIONS: Cell<bool> = const { Cell::new(false) };
        static LARGEST_ALLOCATION: Cell<usize> = const { Cell::new(0) };
    }

    impl FrameAllocationObserver {
        fn observe(size: usize) {
            let _ = OBSERVE_ALLOCATIONS.try_with(|enabled| {
                if enabled.get() {
                    LARGEST_ALLOCATION.with(|largest| largest.set(largest.get().max(size)));
                }
            });
        }
    }

    unsafe impl GlobalAlloc for FrameAllocationObserver {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            let allocation = unsafe { System.alloc(layout) };
            Self::observe(layout.size());
            allocation
        }

        unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
            let allocation = unsafe { System.alloc_zeroed(layout) };
            Self::observe(layout.size());
            allocation
        }

        unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, size: usize) -> *mut u8 {
            let allocation = unsafe { System.realloc(pointer, layout, size) };
            Self::observe(size);
            allocation
        }

        unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
            unsafe { System.dealloc(pointer, layout) };
        }
    }

    struct AllocationObservation;

    impl AllocationObservation {
        fn start() -> Self {
            OBSERVE_ALLOCATIONS.with(|enabled| enabled.set(true));
            LARGEST_ALLOCATION.with(|largest| largest.set(0));
            Self
        }

        fn reset(&self) {
            LARGEST_ALLOCATION.with(|largest| largest.set(0));
        }

        fn largest_request(&self) -> usize {
            LARGEST_ALLOCATION.with(Cell::get)
        }
    }

    impl Drop for AllocationObservation {
        fn drop(&mut self) {
            OBSERVE_ALLOCATIONS.with(|enabled| enabled.set(false));
        }
    }

    #[test]
    fn mapped_cursor_keeps_original_file_alive_after_path_replacement() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.log");
        let replacement = dir.path().join("replacement.log");
        let payload = b"original mapped payload";
        let mut writer = FramedLogWriter::open(&path, FsyncPolicy::Always).unwrap();
        writer.append(7, payload).unwrap();
        writer.sync().unwrap();
        let mut owned_cursor = FramedLogCursor::open(&path).unwrap();
        let owned = owned_cursor.next_frame().unwrap().unwrap();
        let mut cursor = FramedLogCursor::open(&path).unwrap();
        let view = cursor.next_mapped_frame().unwrap().unwrap();

        let mut replacement_writer =
            FramedLogWriter::open(&replacement, FsyncPolicy::Always).unwrap();
        replacement_writer
            .append(8, b"replacement payload")
            .unwrap();
        replacement_writer.sync().unwrap();
        std::fs::rename(&replacement, &path).unwrap();
        drop(cursor);

        assert_eq!(view.seq, owned.seq);
        assert_eq!(view.payload(), owned.payload.as_slice());
        assert_eq!(FramedLogReader::read_frames(&path, 0).unwrap()[0].seq, 8);
        assert_eq!(view.payload(), payload);
    }

    #[test]
    fn mapped_cursor_uses_open_length_and_reread_does_not_advance() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.log");
        let mut writer = FramedLogWriter::open(&path, FsyncPolicy::Always).unwrap();
        writer.append(1, b"prefix").unwrap();
        writer.sync().unwrap();
        let mut cursor = FramedLogCursor::open(&path).unwrap();
        let start = cursor.byte_offset();
        let view = cursor.next_mapped_frame().unwrap().unwrap();
        let next = cursor.byte_offset();
        writer.append(2, b"later append").unwrap();
        writer.sync().unwrap();

        assert_eq!(view.seq, 1);
        assert_eq!(view.payload(), b"prefix");
        fn requires_borrowed_payload(_: &[u8]) {}
        requires_borrowed_payload(view.payload());
        assert!(cursor.next_mapped_frame().unwrap().is_none());
        let reread = cursor.reread_mapped_frame_at(start).unwrap().unwrap();
        assert_eq!(reread.seq, 1);
        assert_eq!(reread.payload(), b"prefix");
        assert_eq!(cursor.byte_offset(), next);
        assert!(cursor.reread_mapped_frame_at(next).unwrap().is_none());
        assert!(cursor.reread_mapped_frame_at(u64::MAX).unwrap().is_none());
    }

    #[test]
    fn mapped_cursor_stops_at_torn_header_payload_and_crc_tail() {
        let dir = tempfile::tempdir().unwrap();
        let header_path = dir.path().join("torn-header.log");
        std::fs::write(&header_path, 4u64.to_le_bytes()).unwrap();
        let mut header_cursor = FramedLogCursor::open(&header_path).unwrap();
        assert!(header_cursor.next_mapped_frame().unwrap().is_none());

        let payload_path = dir.path().join("torn-payload.log");
        let mut payload_writer = FramedLogWriter::open(&payload_path, FsyncPolicy::Always).unwrap();
        payload_writer.append(1, b"clean prefix").unwrap();
        payload_writer.sync().unwrap();
        let mut tail = OpenOptions::new().append(true).open(&payload_path).unwrap();
        tail.write_all(&2u64.to_le_bytes()).unwrap();
        tail.write_all(&3u32.to_le_bytes()).unwrap();
        tail.write_all(&0u32.to_le_bytes()).unwrap();
        tail.write_all(b"to").unwrap();
        tail.sync_all().unwrap();
        let mut payload_cursor = FramedLogCursor::open(&payload_path).unwrap();
        assert_eq!(payload_cursor.next_mapped_frame().unwrap().unwrap().seq, 1);
        assert!(payload_cursor.next_mapped_frame().unwrap().is_none());

        let crc_path = dir.path().join("crc.log");
        let mut crc_writer = FramedLogWriter::open(&crc_path, FsyncPolicy::Always).unwrap();
        crc_writer.append(3, b"bad crc payload").unwrap();
        crc_writer.sync().unwrap();
        let mut corrupt = OpenOptions::new().write(true).open(&crc_path).unwrap();
        corrupt.seek(SeekFrom::Start(HEADER_LEN as u64)).unwrap();
        corrupt.write_all(b"!").unwrap();
        corrupt.sync_all().unwrap();
        let mut crc_cursor = FramedLogCursor::open(&crc_path).unwrap();
        assert!(crc_cursor.next_mapped_frame().unwrap().is_none());
    }

    #[test]
    fn mapped_cursor_rejects_oversized_and_overflow_offsets_without_owned_payload() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("oversized.log");
        let mut file = File::create(&path).unwrap();
        file.write_all(&9u64.to_le_bytes()).unwrap();
        file.write_all(&u32::MAX.to_le_bytes()).unwrap();
        file.write_all(&0u32.to_le_bytes()).unwrap();
        file.sync_all().unwrap();
        let mut cursor = FramedLogCursor::open(&path).unwrap();
        let error = cursor.next_mapped_frame().unwrap_err();
        assert!(error.to_string().contains("oversized legacy log frame"));
        assert!(cursor.reread_mapped_frame_at(u64::MAX).unwrap().is_none());
    }

    #[test]
    fn mapped_cursor_never_allocates_a_payload_sized_transient_buffer() {
        const PAYLOAD_BYTES: usize = 2 * 1024 * 1024;
        const MAX_ALLOWED_REQUEST: usize = 64 * 1024;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("large.log");
        let payload = vec![0x5a; PAYLOAD_BYTES];
        let mut writer = FramedLogWriter::open(&path, FsyncPolicy::Always).unwrap();
        writer.append(44, &payload).unwrap();
        writer.sync().unwrap();
        drop(writer);
        let mut cursor = FramedLogCursor::open(&path).unwrap();
        let start = cursor.byte_offset();

        let observation = AllocationObservation::start();
        let view = cursor.next_mapped_frame().unwrap().unwrap();
        assert_eq!(view.seq, 44);
        assert_eq!(view.payload(), payload.as_slice());
        let next_largest = observation.largest_request();
        drop(view);
        observation.reset();
        let reread = cursor.reread_mapped_frame_at(start).unwrap().unwrap();
        assert_eq!(reread.seq, 44);
        assert_eq!(reread.payload(), payload.as_slice());
        let reread_largest = observation.largest_request();
        drop(reread);
        drop(observation);

        assert!(next_largest <= MAX_ALLOWED_REQUEST);
        assert!(reread_largest <= MAX_ALLOWED_REQUEST);
        assert!(next_largest < PAYLOAD_BYTES);
        assert!(reread_largest < PAYLOAD_BYTES);
    }

    #[test]
    fn cursor_rereads_pinned_frame_after_compaction_without_advancing() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.log");
        let mut log = FramedLogWriter::open(&path, FsyncPolicy::Always).unwrap();
        log.append(10, b"before-compaction").unwrap();
        log.append(20, b"next-original-frame").unwrap();
        log.sync().unwrap();
        let mut cursor = FramedLogCursor::open(&path).unwrap();
        let start = cursor.byte_offset();
        let first = cursor.next_frame().unwrap().unwrap();
        let next = cursor.byte_offset();
        log.truncate_through(10).unwrap();
        assert_eq!(FramedLogReader::read_frames(&path, 0).unwrap()[0].seq, 20);
        assert_eq!(
            cursor.reread_frame_at(start).unwrap(),
            Some(first),
            "capacity retry must reread the original inode and exact frame"
        );
        assert_eq!(
            cursor.byte_offset(),
            next,
            "reread must leave normal replay progression unchanged"
        );
        assert_eq!(cursor.next_frame().unwrap().unwrap().seq, 20);
        assert!(cursor.next_frame().unwrap().is_none());
    }

    #[test]
    fn cursor_reread_keeps_initial_length_and_validates_crc() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.log");
        let mut log = FramedLogWriter::open(&path, FsyncPolicy::Always).unwrap();
        log.append(1, b"one").unwrap();
        log.sync().unwrap();
        let mut cursor = FramedLogCursor::open(&path).unwrap();
        cursor.next_frame().unwrap().unwrap();
        let old_end = cursor.byte_offset();
        log.append(2, b"new-tail").unwrap();
        log.sync().unwrap();
        assert!(
            cursor.reread_frame_at(old_end).unwrap().is_none(),
            "reread must not consume bytes appended after the replay cut"
        );
        assert!(cursor.reread_frame_at(u64::MAX).unwrap().is_none());
        let mut corrupt = OpenOptions::new().write(true).open(&path).unwrap();
        corrupt.seek(SeekFrom::Start(HEADER_LEN as u64)).unwrap();
        corrupt.write_all(b"bad").unwrap();
        corrupt.sync_all().unwrap();
        assert!(
            cursor.reread_frame_at(0).unwrap().is_none(),
            "reread must validate the frame instead of returning stale bytes"
        );
        assert_eq!(cursor.byte_offset(), old_end);
    }

    #[test]
    fn append_replay_truncate_and_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.log");
        let mut log = FramedLogWriter::open(&path, FsyncPolicy::Always).unwrap();
        log.append(1, b"one").unwrap();
        log.append(2, b"two").unwrap();
        log.append(3, b"three").unwrap();
        log.truncate_through(1).unwrap();
        log.append(4, b"four").unwrap();
        log.sync().unwrap();

        let frames = FramedLogReader::read_frames(&path, 0).unwrap();
        let seqs: Vec<u64> = frames.iter().map(|frame| frame.seq).collect();
        assert_eq!(seqs, vec![2, 3, 4]);
    }

    #[test]
    fn torn_tail_replays_prefix_and_open_truncates() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.log");
        let mut log = FramedLogWriter::open(&path, FsyncPolicy::Always).unwrap();
        log.append(1, b"one").unwrap();
        log.append(2, b"two").unwrap();
        log.sync().unwrap();
        let good_len = std::fs::metadata(&path).unwrap().len();
        {
            let mut file = OpenOptions::new().append(true).open(&path).unwrap();
            file.write_all(&99u64.to_le_bytes()).unwrap();
            file.sync_all().unwrap();
        }
        assert_eq!(FramedLogReader::read_frames(&path, 0).unwrap().len(), 2);
        let _ = FramedLogWriter::open(&path, FsyncPolicy::Always).unwrap();
        assert_eq!(std::fs::metadata(&path).unwrap().len(), good_len);
    }

    #[test]
    fn strict_sync_persists_file_and_parent_directory() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.log");
        let mut log = FramedLogWriter::open(&path, FsyncPolicy::Always).unwrap();
        log.append(1, b"one").unwrap();
        log.sync_strict().unwrap();
        assert_eq!(FramedLogReader::read_frames(&path, 0).unwrap().len(), 1);
    }

    #[test]
    fn bounded_reader_pages_without_returning_skipped_frames() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.log");
        let mut log = FramedLogWriter::open(&path, FsyncPolicy::Always).unwrap();
        for sequence in 1..=5 {
            log.append(sequence, format!("frame-{sequence}").as_bytes())
                .unwrap();
        }
        log.sync().unwrap();

        let page = FramedLogReader::read_frames_bounded(&path, 2, 2).unwrap();
        assert_eq!(
            page.iter().map(|frame| frame.seq).collect::<Vec<_>>(),
            [3, 4]
        );
    }

    const LARGE_PAYLOAD_BYTES: usize = MAX_FRAME_PAYLOAD_BYTES + 1;

    #[test]
    fn large_mapped_api_retains_a_valid_legacy_frame_without_large_read_allocation() {
        const MAX_ALLOWED_REQUEST: usize = 64 * 1024;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.log");
        let payload = vec![0x5a; LARGE_PAYLOAD_BYTES];
        let mut log = FramedLogWriter::open(&path, FsyncPolicy::Always).unwrap();
        assert!(log.append(2, &payload).is_err());
        log.append(1, b"discarded prefix").unwrap();
        log.append_large_payload(2, &payload).unwrap();
        log.sync().unwrap();
        let mut pinned_cursor = FramedLogCursor::open(&path).unwrap();

        let mut default_cursor = FramedLogCursor::open(&path).unwrap();
        assert_eq!(default_cursor.next_mapped_frame().unwrap().unwrap().seq, 1);
        assert!(default_cursor.next_mapped_frame().is_err());

        let mut cursor = FramedLogCursor::open(&path).unwrap();
        assert_eq!(cursor.next_large_mapped_frame().unwrap().unwrap().seq, 1);
        let large_offset = cursor.byte_offset();
        let observation = AllocationObservation::start();
        let large = cursor.next_large_mapped_frame().unwrap().unwrap();
        let largest_request = observation.largest_request();
        assert_eq!(large.seq, 2);
        assert_eq!(large.payload(), payload.as_slice());
        assert!(largest_request <= MAX_ALLOWED_REQUEST);
        drop(large);
        drop(observation);

        log.truncate_through_mapped(1).unwrap();
        log.append(3, b"post-compaction append").unwrap();
        log.sync().unwrap();
        let reread = cursor
            .reread_large_mapped_frame_at(large_offset)
            .unwrap()
            .unwrap();
        assert_eq!(reread.seq, 2);
        assert_eq!(reread.payload(), payload.as_slice());
        drop(reread);

        let mut compacted = FramedLogCursor::open(&path).unwrap();
        let retained = compacted.next_large_mapped_frame().unwrap().unwrap();
        assert_eq!(retained.seq, 2);
        assert_eq!(retained.payload(), payload.as_slice());
        let appended = compacted.next_large_mapped_frame().unwrap().unwrap();
        assert_eq!(appended.seq, 3);
        assert_eq!(appended.payload(), b"post-compaction append");
        assert!(compacted.next_large_mapped_frame().unwrap().is_none());

        assert_eq!(
            pinned_cursor
                .next_large_mapped_frame()
                .unwrap()
                .unwrap()
                .seq,
            1
        );
        assert_eq!(
            pinned_cursor
                .next_large_mapped_frame()
                .unwrap()
                .unwrap()
                .seq,
            2
        );
        assert!(pinned_cursor.next_large_mapped_frame().unwrap().is_none());
    }

    #[test]
    fn large_mapped_api_reports_incomplete_and_bad_crc_without_changing_the_file() {
        let dir = tempfile::tempdir().unwrap();
        let incomplete_path = dir.path().join("incomplete.log");
        let mut incomplete = File::create(&incomplete_path).unwrap();
        incomplete.write_all(&9u64.to_le_bytes()).unwrap();
        incomplete
            .write_all(&(LARGE_PAYLOAD_BYTES as u32).to_le_bytes())
            .unwrap();
        incomplete.write_all(&0u32.to_le_bytes()).unwrap();
        incomplete.sync_all().unwrap();
        let before_incomplete = std::fs::metadata(&incomplete_path).unwrap().len();
        let mut incomplete_cursor = FramedLogCursor::open(&incomplete_path).unwrap();
        let error = incomplete_cursor.next_large_mapped_frame().unwrap_err();
        assert!(error.to_string().contains("oversized legacy log frame"));
        assert_eq!(
            std::fs::metadata(&incomplete_path).unwrap().len(),
            before_incomplete
        );

        let bad_crc_path = dir.path().join("bad-crc.log");
        let mut bad_crc = File::create(&bad_crc_path).unwrap();
        bad_crc.write_all(&10u64.to_le_bytes()).unwrap();
        bad_crc
            .write_all(&(LARGE_PAYLOAD_BYTES as u32).to_le_bytes())
            .unwrap();
        bad_crc.write_all(&1u32.to_le_bytes()).unwrap();
        bad_crc
            .set_len(HEADER_LEN as u64 + LARGE_PAYLOAD_BYTES as u64)
            .unwrap();
        bad_crc.sync_all().unwrap();
        let before_bad_crc = std::fs::metadata(&bad_crc_path).unwrap().len();
        let mut bad_crc_cursor = FramedLogCursor::open(&bad_crc_path).unwrap();
        let error = bad_crc_cursor.next_large_mapped_frame().unwrap_err();
        assert!(error.to_string().contains("failed validation"));
        assert_eq!(
            std::fs::metadata(&bad_crc_path).unwrap().len(),
            before_bad_crc
        );
    }

    #[test]
    fn mapped_truncation_does_not_publish_when_a_large_frame_fails_validation() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.log");
        let mut log = FramedLogWriter::open(&path, FsyncPolicy::Always).unwrap();
        log.append(1, b"original prefix").unwrap();
        log.sync().unwrap();

        let mut corrupt = OpenOptions::new().append(true).open(&path).unwrap();
        corrupt.write_all(&2u64.to_le_bytes()).unwrap();
        corrupt
            .write_all(&(LARGE_PAYLOAD_BYTES as u32).to_le_bytes())
            .unwrap();
        corrupt.write_all(&1u32.to_le_bytes()).unwrap();
        corrupt
            .set_len(std::fs::metadata(&path).unwrap().len() + LARGE_PAYLOAD_BYTES as u64)
            .unwrap();
        corrupt.sync_all().unwrap();
        let before = std::fs::metadata(&path).unwrap().len();

        let error = log.truncate_through_mapped(0).unwrap_err();
        assert!(error.to_string().contains("failed validation"));
        assert_eq!(std::fs::metadata(&path).unwrap().len(), before);
        let mut cursor = FramedLogCursor::open(&path).unwrap();
        assert_eq!(cursor.next_large_mapped_frame().unwrap().unwrap().seq, 1);
        assert!(cursor.next_large_mapped_frame().is_err());
    }

    fn replay_sequences(path: &Path) -> Vec<u64> {
        FramedLogReader::read_frames(path, 0)
            .unwrap()
            .into_iter()
            .map(|frame| frame.seq)
            .collect()
    }

    #[test]
    fn synchronous_mapped_trim_retains_readable_appender_and_reopens() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("synchronous.log");
        let mut log = FramedLogWriter::open(&path, FsyncPolicy::Always).unwrap();
        log.append(1, b"covered").unwrap();
        log.append(2, b"retained").unwrap();
        log.truncate_mapped_in_place(1).unwrap();
        assert_eq!(replay_sequences(&path), vec![2]);

        let mut retained = log.file.get_ref().try_clone().unwrap();
        retained.seek(SeekFrom::Start(0)).unwrap();
        let mut header = [0u8; HEADER_LEN];
        retained.read_exact(&mut header).unwrap();
        assert_eq!(u64::from_le_bytes(header[..8].try_into().unwrap()), 2);
        log.append(3, b"after publication").unwrap();
        drop(retained);
        drop(log);
        assert_eq!(replay_sequences(&path), vec![2, 3]);

        let mut reopened = FramedLogWriter::open(&path, FsyncPolicy::Always).unwrap();
        reopened.append(4, b"after reopen").unwrap();
        drop(reopened);
        assert_eq!(replay_sequences(&path), vec![2, 3, 4]);
    }

    #[test]
    #[cfg(unix)]
    fn mapped_trim_plan_ready_prefix_and_exact_suffix_survive_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.log");
        let mut log = FramedLogWriter::open(&path, FsyncPolicy::Always).unwrap();
        for sequence in 1..=3 {
            log.append(sequence, &sequence.to_le_bytes()).unwrap();
        }

        let mut plan = log.begin_trim_mapped(1).unwrap();
        plan.copy_stable_prefix().unwrap();
        log.append(4, b"late suffix").unwrap();
        log.finish_trim_mapped(plan).unwrap();
        drop(log);

        assert_eq!(replay_sequences(&path), vec![2, 3, 4]);
        let mut reopened = FramedLogWriter::open(&path, FsyncPolicy::Always).unwrap();
        reopened.append(5, b"after reopen").unwrap();
        drop(reopened);
        assert_eq!(replay_sequences(&path), vec![2, 3, 4, 5]);
    }

    #[test]
    #[cfg(unix)]
    fn mapped_trim_plan_requires_ready_full_prefix_before_publication() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.log");
        let mut log = FramedLogWriter::open(&path, FsyncPolicy::Always).unwrap();
        log.append(1, b"covered").unwrap();
        log.append(2, b"retained").unwrap();

        let plan = log.begin_trim_mapped(1).unwrap();
        assert!(log.finish_trim_mapped(plan).is_err());
        drop(log);
        assert_eq!(replay_sequences(&path), vec![1, 2]);
    }

    #[test]
    #[cfg(unix)]
    fn mapped_trim_plan_busy_preserves_active_fixed_temp_and_drop_releases_it() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.log");
        let mut log = FramedLogWriter::open(&path, FsyncPolicy::Always).unwrap();
        log.append(1, b"covered").unwrap();
        log.append(2, b"retained").unwrap();

        let plan = log.begin_trim_mapped(1).unwrap();
        let tmp = log.compact_tmp_path();
        let mut marker = OpenOptions::new().append(true).open(&tmp).unwrap();
        marker.write_all(b"active-plan-owned-temp").unwrap();
        marker.sync_all().unwrap();
        let before = std::fs::read(&tmp).unwrap();
        assert!(log.begin_trim_mapped(1).unwrap_err().to_string().contains("Busy"));
        assert!(log.truncate_through(1).unwrap_err().to_string().contains("Busy"));
        assert_eq!(std::fs::read(&tmp).unwrap(), before);

        drop(plan);
        log.truncate_through(1).unwrap();
        log.append(3, b"after generic trim").unwrap();
        let mut replacement = log.begin_trim_mapped(1).unwrap();
        replacement.copy_stable_prefix().unwrap();
        log.finish_trim_mapped(replacement).unwrap();
        assert_eq!(replay_sequences(&path), vec![2, 3]);
    }

    #[test]
    #[cfg(unix)]
    fn mapped_trim_plan_rejects_wrong_writer_revision_and_replaced_inode() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.log");
        let mut log = FramedLogWriter::open(&path, FsyncPolicy::Always).unwrap();
        log.append(1, b"covered").unwrap();
        log.append(2, b"retained").unwrap();

        let mut wrong_writer_plan = log.begin_trim_mapped(1).unwrap();
        wrong_writer_plan.copy_stable_prefix().unwrap();
        let mut other = FramedLogWriter::open(&path, FsyncPolicy::Always).unwrap();
        assert!(other.finish_trim_mapped(wrong_writer_plan).is_err());
        assert_eq!(replay_sequences(&path), vec![1, 2]);

        let mut revision_plan = log.begin_trim_mapped(1).unwrap();
        revision_plan.copy_stable_prefix().unwrap();
        log.trim_revision += 1;
        assert!(log.finish_trim_mapped(revision_plan).is_err());
        assert_eq!(replay_sequences(&path), vec![1, 2]);

        let mut inode_plan = log.begin_trim_mapped(1).unwrap();
        inode_plan.copy_stable_prefix().unwrap();
        let replacement = dir.path().join("replacement.log");
        std::fs::write(&replacement, []).unwrap();
        std::fs::rename(&replacement, &path).unwrap();
        let replacement_bytes = std::fs::read(&path).unwrap();
        assert!(log.finish_trim_mapped(inode_plan).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), replacement_bytes);
    }

    #[test]
    #[cfg(unix)]
    fn mapped_trim_plan_streams_large_payload_in_multiple_fixed_chunks() {
        const CHUNK: usize = 64 * 1024;
        const PAYLOAD_BYTES: usize = CHUNK * 3 + 17;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("large.log");
        let payload = vec![0x5a; PAYLOAD_BYTES];
        let mut log = FramedLogWriter::open(&path, FsyncPolicy::Always).unwrap();
        log.append(1, b"covered").unwrap();
        log.append_large_payload(2, &payload).unwrap();

        let mut plan = log.begin_trim_mapped(1).unwrap();
        let observation = AllocationObservation::start();
        plan.copy_stable_prefix().unwrap();
        let largest_request = observation.largest_request();
        drop(observation);
        log.finish_trim_mapped(plan).unwrap();

        assert!(largest_request <= CHUNK);
        assert_eq!(replay_sequences(&path), vec![2]);
    }

    #[test]
    #[cfg(unix)]
    fn mapped_trim_plan_invalid_prefix_or_suffix_never_replaces_old_aof() {
        let dir = tempfile::tempdir().unwrap();
        let prefix_path = dir.path().join("prefix.log");
        let mut prefix = FramedLogWriter::open(&prefix_path, FsyncPolicy::Always).unwrap();
        prefix.append(1, b"covered").unwrap();
        prefix.append(2, b"retained").unwrap();
        prefix.sync().unwrap();
        let mut corrupt = OpenOptions::new().write(true).open(&prefix_path).unwrap();
        corrupt.seek(SeekFrom::Start(HEADER_LEN as u64)).unwrap();
        corrupt.write_all(b"!").unwrap();
        corrupt.sync_all().unwrap();
        let before_prefix = std::fs::metadata(&prefix_path).unwrap().len();
        let mut prefix_plan = prefix.begin_trim_mapped(1).unwrap();
        assert!(prefix_plan.copy_stable_prefix().is_err());
        assert!(prefix_plan.copy_stable_prefix().is_err());
        assert!(prefix.finish_trim_mapped(prefix_plan).is_err());
        assert_eq!(std::fs::metadata(&prefix_path).unwrap().len(), before_prefix);

        let torn_prefix_path = dir.path().join("torn-prefix.log");
        let mut torn_prefix = FramedLogWriter::open(&torn_prefix_path, FsyncPolicy::Always).unwrap();
        torn_prefix.append(1, b"covered").unwrap();
        torn_prefix.append(2, b"retained").unwrap();
        torn_prefix.sync().unwrap();
        let mut torn_prefix_bytes = OpenOptions::new().append(true).open(&torn_prefix_path).unwrap();
        torn_prefix_bytes.write_all(&3u64.to_le_bytes()).unwrap();
        torn_prefix_bytes.sync_all().unwrap();
        let mut torn_prefix_plan = torn_prefix.begin_trim_mapped(3).unwrap();
        assert!(torn_prefix_plan.copy_stable_prefix().is_err());
        assert!(torn_prefix.finish_trim_mapped(torn_prefix_plan).is_err());
        drop(torn_prefix);
        assert_eq!(replay_sequences(&torn_prefix_path), vec![1, 2]);

        let suffix_path = dir.path().join("suffix.log");
        let mut suffix = FramedLogWriter::open(&suffix_path, FsyncPolicy::Always).unwrap();
        suffix.append(1, b"covered").unwrap();
        suffix.append(2, b"retained").unwrap();
        let mut suffix_plan = suffix.begin_trim_mapped(1).unwrap();
        suffix_plan.copy_stable_prefix().unwrap();
        let mut corrupt_suffix = OpenOptions::new().append(true).open(&suffix_path).unwrap();
        corrupt_suffix.write_all(&3u64.to_le_bytes()).unwrap();
        corrupt_suffix.write_all(&4u32.to_le_bytes()).unwrap();
        corrupt_suffix.write_all(&0u32.to_le_bytes()).unwrap();
        corrupt_suffix.write_all(b"crc!").unwrap();
        corrupt_suffix.sync_all().unwrap();
        let before_suffix = std::fs::metadata(&suffix_path).unwrap().len();
        assert!(suffix.finish_trim_mapped(suffix_plan).is_err());
        assert_eq!(std::fs::metadata(&suffix_path).unwrap().len(), before_suffix);
        drop(suffix);
        assert_eq!(replay_sequences(&suffix_path), vec![1, 2]);
    }

    #[test]
    #[cfg(unix)]
    fn mapped_trim_plan_torn_suffix_keeps_exact_original_bytes() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("torn-suffix.log");
        let mut log = FramedLogWriter::open(&path, FsyncPolicy::Always).unwrap();
        log.append(1, b"covered").unwrap();
        log.append(2, b"retained").unwrap();
        let mut plan = log.begin_trim_mapped(1).unwrap();
        plan.copy_stable_prefix().unwrap();
        log.append(3, b"acknowledged suffix").unwrap();
        let mut tail = OpenOptions::new().append(true).open(&path).unwrap();
        tail.write_all(&4u64.to_le_bytes()).unwrap();
        tail.sync_all().unwrap();
        let before = std::fs::read(&path).unwrap();
        assert!(log.finish_trim_mapped(plan).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), before);
        assert_eq!(replay_sequences(&path), vec![1, 2, 3]);
    }

    #[test]
    #[cfg(unix)]
    fn mapped_trim_failed_begin_and_abandoned_temp_do_not_strand_ownership() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("begin.log");
        let mut log = FramedLogWriter::open(&path, FsyncPolicy::Always).unwrap();
        log.append(1, b"covered").unwrap();
        let tmp = log.compact_tmp_path();
        std::fs::create_dir(&tmp).unwrap();
        assert!(log.begin_trim_mapped(1).is_err());
        assert!(log.trim_active.lock().unwrap().is_none());
        std::fs::remove_dir(&tmp).unwrap();
        std::fs::write(&tmp, b"abandoned prior attempt").unwrap();
        let mut plan = log.begin_trim_mapped(1).unwrap();
        assert!(std::fs::read(&tmp).unwrap().is_empty());
        plan.copy_stable_prefix().unwrap();
        log.finish_trim_mapped(plan).unwrap();
        assert!(!tmp.exists());
        assert!(log.trim_active.lock().unwrap().is_none());
        assert!(replay_sequences(&path).is_empty());
    }

    #[test]
    #[cfg(unix)]
    fn mapped_trim_rejects_changed_temp_and_repeated_ready_copy() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("temp-identity.log");
        let mut log = FramedLogWriter::open(&path, FsyncPolicy::Always).unwrap();
        log.append(1, b"covered").unwrap();
        log.append(2, b"retained").unwrap();
        let before = std::fs::read(&path).unwrap();
        let mut plan = log.begin_trim_mapped(1).unwrap();
        plan.copy_stable_prefix().unwrap();
        assert!(plan.copy_stable_prefix().is_err());
        assert!(log.finish_trim_mapped(plan).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), before);
        let mut plan = log.begin_trim_mapped(1).unwrap();
        plan.copy_stable_prefix().unwrap();
        let replacement = dir.path().join("replacement.tmp");
        std::fs::write(&replacement, b"unrelated inode").unwrap();
        std::fs::rename(&replacement, log.compact_tmp_path()).unwrap();
        assert!(log.finish_trim_mapped(plan).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), before);
        log.append(3, b"after rejected temp").unwrap();
        assert_eq!(replay_sequences(&path), vec![1, 2, 3]);
    }

    #[test]
    #[cfg(unix)]
    fn mapped_trim_plan_faults_preserve_pre_and_post_rename_contracts() {
        for point in [
            TrimFaultPoint::PrefixTempSync,
            TrimFaultPoint::SuffixTempSync,
        ] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("state.log");
            let mut log = FramedLogWriter::open(&path, FsyncPolicy::Always).unwrap();
            log.append(1, b"covered").unwrap();
            log.append(2, b"retained").unwrap();
            let faults = log.trim_faults_for_test();
            faults.fail_once(point);
            let mut plan = log.begin_trim_mapped(1).unwrap();
            let result = plan.copy_stable_prefix();
            if point == TrimFaultPoint::SuffixTempSync {
                result.unwrap();
                log.append(3, b"suffix").unwrap();
                assert!(log.finish_trim_mapped(plan).is_err());
            } else {
                assert!(result.is_err());
                drop(plan);
                assert!(log.trim_active.lock().unwrap().is_none());
                assert!(!log.compact_tmp_path().exists());
                let retry = log.begin_trim_mapped(1).unwrap();
                drop(retry);
            }
            drop(log);
            let expected = if point == TrimFaultPoint::SuffixTempSync {
                vec![1, 2, 3]
            } else {
                vec![1, 2]
            };
            assert_eq!(replay_sequences(&path), expected);
        }

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("parent-sync.log");
        let mut log = FramedLogWriter::open(&path, FsyncPolicy::Always).unwrap();
        log.append(1, b"covered").unwrap();
        log.append(2, b"retained").unwrap();
        let faults = log.trim_faults_for_test();
        faults.fail_once(TrimFaultPoint::ParentSyncAfterRename);
        let mut plan = log.begin_trim_mapped(1).unwrap();
        plan.copy_stable_prefix().unwrap();
        assert!(log.finish_trim_mapped(plan).is_err());
        log.append(3, b"later append").unwrap();
        drop(log);
        assert_eq!(replay_sequences(&path), vec![2, 3]);
    }
}
// CODEGEN-END

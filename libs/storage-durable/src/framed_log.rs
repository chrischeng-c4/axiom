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
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};

use crate::{strict_sync_parent_dir, sync_parent_dir, FsyncPolicy};

const HEADER_LEN: usize = 16;

/// Largest payload accepted by the shared framed-log format.
///
/// This bound prevents a corrupt or hostile header from forcing a multi-GiB
/// allocation during recovery. Callers must split larger logical records.
pub const MAX_FRAME_PAYLOAD_BYTES: usize = 64 * 1024 * 1024;

/// One validated log frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogFrame {
    pub seq: u64,
    pub payload: Vec<u8>,
}

/// CRC-framed append log with clean torn-tail recovery.
pub struct FramedLogWriter {
    path: PathBuf,
    file: BufWriter<File>,
    policy: FsyncPolicy,
    last_sync: Instant,
    sync_every: Duration,
    dirty: bool,
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
        })
    }

    pub fn append(&mut self, seq: u64, payload: &[u8]) -> Result<()> {
        let len = checked_payload_len(payload.len())?;
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
        self.flush()?;
        let mut frames = FramedLogCursor::open(&self.path)?;
        let tmp = self.compact_tmp_path();
        let _ = std::fs::remove_file(&tmp);
        {
            let mut dst = BufWriter::new(
                OpenOptions::new()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .open(&tmp)
                    .with_context(|| format!("create log compaction temp {}", tmp.display()))?,
            );
            while let Some(frame) = frames.next_frame()? {
                if frame.seq > through {
                    write_frame(&mut dst, frame.seq, &frame.payload)?;
                }
            }
            dst.flush().context("flush log compaction temp")?;
            dst.get_ref()
                .sync_all()
                .context("fsync log compaction temp")?;
        }
        std::fs::rename(&tmp, &self.path).with_context(|| {
            format!(
                "commit log compaction {} -> {}",
                tmp.display(),
                self.path.display()
            )
        })?;
        sync_parent_dir(&self.path)?;
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .with_context(|| format!("re-open compacted log {}", self.path.display()))?;
        self.file = BufWriter::new(file);
        self.dirty = false;
        self.last_sync = Instant::now();
        Ok(())
    }

    fn compact_tmp_path(&self) -> PathBuf {
        let mut tmp = self.path.as_os_str().to_os_string();
        tmp.push(".compact.tmp");
        tmp.into()
    }
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

fn checked_payload_len(len: usize) -> Result<u32> {
    if len > MAX_FRAME_PAYLOAD_BYTES {
        anyhow::bail!("log payload {len} bytes exceeds maximum {MAX_FRAME_PAYLOAD_BYTES} bytes");
    }
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
}
// CODEGEN-END

// CODEGEN-BEGIN
//! Append log whose tail is allowed to be garbage.
//!
//! A frame is a 16-byte header -- `seq: u64 LE`, `len: u32 LE`, `crc32: u32 LE`
//! -- followed by `len` payload bytes. **The CRC covers the payload only, never
//! the header.** A corrupted header cannot announce itself as corrupt: it is
//! read as some plausible `(seq, len)` pair and rejected only because `len` runs
//! past EOF or because the payload it points at fails its CRC. That is the whole
//! torn-tail rule -- the first frame failing any check ends the log, and every
//! byte after it is discarded, unexamined.
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

use anyhow::{Context, Result};

use crate::{strict_sync_parent_dir, sync_parent_dir, FsyncPolicy};

const HEADER_LEN: usize = 16;

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
        let len = u32::try_from(payload.len()).context("log payload too large for u32 len")?;
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
        let frames = FramedLogReader::read_frames(&self.path, 0)?;
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
            for frame in frames.into_iter().filter(|frame| frame.seq > through) {
                write_frame(&mut dst, frame.seq, &frame.payload)?;
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

/// Reader for CRC-framed append logs.
pub struct FramedLogReader;

impl FramedLogReader {
    pub fn replay(
        path: impl AsRef<Path>,
        from_seq: u64,
        mut apply: impl FnMut(LogFrame),
    ) -> Result<u64> {
        let mut max_seq = 0u64;
        for frame in Self::read_frames(path, from_seq)? {
            max_seq = max_seq.max(frame.seq);
            apply(frame);
        }
        Ok(max_seq)
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
            let frame_end = off + HEADER_LEN as u64 + len;
            if frame_end > total {
                break;
            }
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
        while let Some((_, _, next)) = read_one_frame(&mut file, total, off, &mut header)? {
            off = next;
        }
        Ok(off)
    }
}

fn write_frame(mut writer: impl Write, seq: u64, payload: &[u8]) -> Result<()> {
    let len = u32::try_from(payload.len()).context("log payload too large for u32 len")?;
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
    let frame_end = off + HEADER_LEN as u64 + len;
    if frame_end > total {
        return Ok(None);
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

---
id: libs-service-durability-src-framed-log-rs
summary: Lossless rust-source-unit coverage for `libs/service-durability/src/framed_log.rs`.
capability_refs:
  - id: shared-service-durability-contract
    role: primary
    gap: shared-service-durability-contract
    claim: shared-service-durability-contract
    coverage: full
    rationale: "The source unit implements CRC-framed append logs with torn-tail recovery."
fill_sections: [overview, source, changes]
---

# Standardized libs/service-durability/src/framed_log.rs

## Overview
<!-- type: overview lang: markdown -->

Lossless rust-source-unit coverage for `libs/service-durability/src/framed_log.rs`.

### Symbols

| Name | Target | Kind | Visibility | Signature |
|------|--------|------|------------|-----------|
| `LogFrame` | libs/service-durability/src/framed_log.rs | struct | pub | LogFrame { seq, payload } |
| `FramedLogWriter` | libs/service-durability/src/framed_log.rs | struct | pub | FramedLogWriter |
| `open` | libs/service-durability/src/framed_log.rs | method | pub | open(path, policy) -> Result<Self> |
| `append` | libs/service-durability/src/framed_log.rs | method | pub | append(&mut self, seq, payload) -> Result<()> |
| `flush` | libs/service-durability/src/framed_log.rs | method | pub | flush(&mut self) -> Result<()> |
| `sync` | libs/service-durability/src/framed_log.rs | method | pub | sync(&mut self) -> Result<()> |
| `maybe_sync` | libs/service-durability/src/framed_log.rs | method | pub | maybe_sync(&mut self) -> Result<()> |
| `truncate_through` | libs/service-durability/src/framed_log.rs | method | pub | truncate_through(&mut self, through) -> Result<()> |
| `FramedLogReader` | libs/service-durability/src/framed_log.rs | struct | pub | FramedLogReader |
| `replay` | libs/service-durability/src/framed_log.rs | method | pub | replay(path, from_seq, apply) -> Result<u64> |
| `read_frames` | libs/service-durability/src/framed_log.rs | method | pub | read_frames(path, from_seq) -> Result<Vec<LogFrame>> |
| `scan_good_end` | libs/service-durability/src/framed_log.rs | method | pub | scan_good_end(path) -> Result<u64> |

## Source
<!-- type: rust-source-unit lang: rust -->

````rust
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use anyhow::{Context, Result};

use crate::{sync_parent_dir, FsyncPolicy};

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
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/service-durability/src/framed_log.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/service-durability/src/framed_log.rs`.
```

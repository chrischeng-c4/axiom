// HANDWRITE-BEGIN gap="missing-generator:logic:255e6322" tracker="1675" reason="Own source-neutral records, enrichment, opaque commit cursors, outcomes, CollectorSource, and linear file/stdin framing."
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use std::path::PathBuf;

use anyhow::{bail, Context, Result};

use crate::AttributeValue;

use super::checkpoint::{CollectorCheckpoint, QuarantineEntry};
use super::cri::CriSource;
use super::{CollectorConfig, SourceSpec};

pub(crate) use service_collector::CommitStats;

#[derive(Clone, Debug, Default)]
pub(crate) struct RecordEnrichment {
    pub(crate) resource: BTreeMap<String, String>,
    pub(crate) attributes: BTreeMap<String, AttributeValue>,
    pub(crate) cloud_logging_coexistence: bool,
}

#[derive(Clone, Debug)]
pub(crate) struct RawRecord {
    pub(crate) source_id: String,
    pub(crate) line: u64,
    pub(crate) offset: u64,
    pub(crate) bytes: Vec<u8>,
    pub(crate) cursor: SourceCursor,
    pub(crate) enrichment: RecordEnrichment,
}

impl service_collector::CollectorRecord for RawRecord {
    type Cursor = SourceCursor;

    fn cursor(&self) -> &Self::Cursor {
        &self.cursor
    }
}

#[derive(Clone, Debug)]
pub(crate) struct SourceRejection {
    pub(crate) entry: QuarantineEntry,
    pub(crate) cursor: SourceCursor,
}

impl service_collector::CollectorRejection for SourceRejection {
    type Cursor = SourceCursor;
    type Entry = QuarantineEntry;

    fn into_parts(self) -> (Self::Entry, Self::Cursor) {
        (self.entry, self.cursor)
    }
}

#[derive(Clone, Debug)]
pub(crate) enum SourceCursor {
    Linear {
        next_offset: u64,
        next_line: u64,
    },
    Cri {
        identity: String,
        next_offset: u64,
        next_line: u64,
        observed_len: u64,
    },
    CriLoss {
        identity: String,
        lost_bytes: u64,
    },
}

pub(crate) type ReadOutcome = service_collector::ReadOutcome<RawRecord, SourceRejection>;

pub(crate) type DynCollectorSource = dyn service_collector::CollectorSource<
    Cursor = SourceCursor,
    Error = anyhow::Error,
    Record = RawRecord,
    Rejection = SourceRejection,
>;

pub(crate) fn open_source(config: &CollectorConfig) -> Result<Box<DynCollectorSource>> {
    match &config.source {
        SourceSpec::File(path) => Ok(Box::new(LinearSource::file(
            path.clone(),
            config.source_id.clone(),
            config.checkpoint_path.clone(),
        )?)),
        SourceSpec::Stdin => Ok(Box::new(LinearSource::stdin(
            config.source_id.clone(),
            config.checkpoint_path.clone(),
        )?)),
        SourceSpec::Cri(cri) => Ok(Box::new(CriSource::open(
            cri.clone(),
            config.checkpoint_path.clone(),
        )?)),
    }
}

enum LinearReader {
    File {
        path: PathBuf,
        reader: BufReader<File>,
    },
    Stdin,
}

struct LinearSource {
    source_id: String,
    checkpoint_path: PathBuf,
    checkpoint: CollectorCheckpoint,
    reader: LinearReader,
    read_offset: u64,
    read_line: u64,
    start_offset: u64,
}

impl LinearSource {
    fn file(path: PathBuf, source_id: String, checkpoint_path: PathBuf) -> Result<Self> {
        let checkpoint = CollectorCheckpoint::load(&checkpoint_path, &source_id)?;
        let metadata = std::fs::metadata(&path)
            .with_context(|| format!("inspect collector source {}", path.display()))?;
        if !metadata.is_file() {
            bail!("collector file source must be a regular file");
        }
        if metadata.len() < checkpoint.offset {
            bail!(
                "collector source was truncated or rotated: size {} is below checkpoint {}; use a new source_id or --cri-root for rotation-aware collection",
                metadata.len(),
                checkpoint.offset
            );
        }
        let mut file = File::open(&path)
            .with_context(|| format!("open collector source {}", path.display()))?;
        file.seek(SeekFrom::Start(checkpoint.offset))
            .with_context(|| format!("seek collector source {}", path.display()))?;
        let read_offset = checkpoint.offset;
        let read_line = checkpoint.line;
        Ok(Self {
            source_id,
            checkpoint_path,
            start_offset: read_offset,
            checkpoint,
            reader: LinearReader::File {
                path,
                reader: BufReader::new(file),
            },
            read_offset,
            read_line,
        })
    }

    fn stdin(source_id: String, checkpoint_path: PathBuf) -> Result<Self> {
        let checkpoint = CollectorCheckpoint::load(&checkpoint_path, &source_id)?;
        let stdin = std::io::stdin();
        discard_acknowledged(&mut stdin.lock(), checkpoint.offset)?;
        let read_offset = checkpoint.offset;
        let read_line = checkpoint.line;
        Ok(Self {
            source_id,
            checkpoint_path,
            start_offset: read_offset,
            checkpoint,
            reader: LinearReader::Stdin,
            read_offset,
            read_line,
        })
    }

    fn read(&mut self, max_bytes: usize) -> Result<BoundedLine> {
        match &mut self.reader {
            LinearReader::File { reader, .. } => read_bounded_line(reader, max_bytes),
            LinearReader::Stdin => read_bounded_line(&mut std::io::stdin().lock(), max_bytes),
        }
    }
}

impl service_collector::CollectorSource for LinearSource {
    type Cursor = SourceCursor;
    type Error = anyhow::Error;
    type Record = RawRecord;
    type Rejection = SourceRejection;

    fn next_record(&mut self, max_bytes: usize) -> Result<ReadOutcome> {
        let start_offset = self.read_offset;
        let read = self.read(max_bytes)?;
        if read.bytes_read == 0 {
            return Ok(ReadOutcome::Exhausted);
        }
        self.read_offset = self
            .read_offset
            .checked_add(read.bytes_read)
            .context("collector byte offset overflow")?;
        self.read_line = self
            .read_line
            .checked_add(1)
            .context("collector line counter overflow")?;
        let cursor = SourceCursor::Linear {
            next_offset: self.read_offset,
            next_line: self.read_line,
        };
        if read.oversized {
            return Ok(ReadOutcome::Rejection(SourceRejection {
                entry: QuarantineEntry::invalid_line(
                    &self.source_id,
                    self.read_line,
                    start_offset,
                    "line_too_large",
                    format!("structured stdout line exceeds {max_bytes} bytes"),
                    &read.preview,
                ),
                cursor,
            }));
        }
        Ok(ReadOutcome::Record(RawRecord {
            source_id: self.source_id.clone(),
            line: self.read_line,
            offset: start_offset,
            bytes: read.preview,
            cursor,
            enrichment: RecordEnrichment::default(),
        }))
    }

    fn commit(&mut self, cursors: &[SourceCursor], stats: CommitStats) -> Result<()> {
        for cursor in cursors {
            match cursor {
                SourceCursor::Linear {
                    next_offset,
                    next_line,
                } => {
                    self.checkpoint.offset = *next_offset;
                    self.checkpoint.line = *next_line;
                }
                SourceCursor::Cri { .. } | SourceCursor::CriLoss { .. } => {
                    bail!("CRI cursor committed to linear source")
                }
            }
        }
        self.checkpoint.accepted = self
            .checkpoint
            .accepted
            .checked_add(stats.accepted)
            .context("collector accepted counter overflow")?;
        self.checkpoint.duplicates = self
            .checkpoint
            .duplicates
            .checked_add(stats.duplicates)
            .context("collector duplicate counter overflow")?;
        self.checkpoint.rejected = self
            .checkpoint
            .rejected
            .checked_add(stats.rejected)
            .context("collector rejected counter overflow")?;
        self.checkpoint.save(&self.checkpoint_path)
    }

    fn refresh(&mut self) -> Result<()> {
        if let LinearReader::File { path, .. } = &self.reader {
            let len = std::fs::metadata(path)
                .with_context(|| format!("inspect collector source {}", path.display()))?
                .len();
            if len < self.read_offset {
                bail!(
                    "collector source was truncated or rotated: size {len} is below read offset {}; use --cri-root for rotation-aware collection",
                    self.read_offset
                );
            }
        }
        Ok(())
    }

    fn progress(&self) -> service_collector::SourceProgress {
        service_collector::SourceProgress {
            start_offset: self.start_offset,
            final_offset: self.checkpoint.offset,
            lost_bytes: 0,
            lost_sources: 0,
        }
    }
}

pub(crate) struct BoundedLine {
    pub(crate) bytes_read: u64,
    pub(crate) preview: Vec<u8>,
    pub(crate) oversized: bool,
}

pub(crate) fn read_bounded_line(
    reader: &mut impl BufRead,
    max_bytes: usize,
) -> Result<BoundedLine> {
    let mut bytes_read = 0_u64;
    let mut preview = Vec::with_capacity(max_bytes.min(8192));
    let mut terminated = false;

    while !terminated {
        let available = reader.fill_buf().context("read collector source")?;
        if available.is_empty() {
            break;
        }
        let consumed = available
            .iter()
            .position(|byte| *byte == b'\n')
            .map(|position| position + 1)
            .unwrap_or(available.len());
        let remaining = max_bytes.saturating_add(1).saturating_sub(preview.len());
        preview.extend_from_slice(&available[..consumed.min(remaining)]);
        bytes_read = bytes_read
            .checked_add(consumed as u64)
            .context("collector line byte count overflow")?;
        terminated = available[..consumed].last() == Some(&b'\n');
        reader.consume(consumed);
    }

    Ok(BoundedLine {
        bytes_read,
        oversized: bytes_read > max_bytes as u64,
        preview,
    })
}

fn discard_acknowledged(reader: &mut impl Read, mut bytes: u64) -> Result<()> {
    let mut buffer = [0_u8; 8192];
    while bytes > 0 {
        let wanted = usize::try_from(bytes.min(buffer.len() as u64)).unwrap();
        let read = reader
            .read(&mut buffer[..wanted])
            .context("discard acknowledged stdin bytes")?;
        if read == 0 {
            bail!(
                "stdin ended before collector checkpoint offset; replay the same source or use a new source_id"
            );
        }
        bytes -= read as u64;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn bounded_reader_discards_oversized_line_and_continues() {
        let source = format!("{}\n{{}}\n", "x".repeat(64));
        let mut reader = Cursor::new(source.into_bytes());
        let oversized = read_bounded_line(&mut reader, 16).unwrap();
        assert!(oversized.oversized);
        assert_eq!(oversized.preview.len(), 17);
        let next = read_bounded_line(&mut reader, 16).unwrap();
        assert!(!next.oversized);
        assert_eq!(next.preview, b"{}\n");
    }

    #[test]
    fn stdin_resume_discards_exact_checkpoint_bytes() {
        let mut source = Cursor::new(b"first\nsecond\n".to_vec());
        discard_acknowledged(&mut source, 6).unwrap();
        let mut rest = String::new();
        source.read_to_string(&mut rest).unwrap();
        assert_eq!(rest, "second\n");
        assert!(discard_acknowledged(&mut Cursor::new(b"short".to_vec()), 9).is_err());
    }
}

// HANDWRITE-END

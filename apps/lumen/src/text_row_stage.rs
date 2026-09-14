//! Bounded external-sort seal for one local Text row.
//!
//! This module is a child of `segment`, so it uses the established private
//! LSEG codec.  It stores sorted, deduplicated term runs on the caller's
//! staging filesystem and merges at fan-in two.  It never builds the field
//! dictionary, a postings map, or a token list in memory.
//!
//! Scratch memory uses one fixed codec/IO area and sixteen payload slots.
//! Sorted-run storage, merge heads, prefix buffers and serialized/compressed
//! blocks use those slots. Skip metadata streams through disk files, so it
//! does not grow the writer heap. An oversized individual term returns an
//! internal workspace requirement for the caller to reserve before retrying.

use super::*;
use anyhow::{anyhow, bail, Context, Result};
use std::cmp::Ordering;
use std::fs::{self, File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering as AtomicOrdering};

static ROW_STAGE_NONCE: AtomicU64 = AtomicU64::new(0);

/// The current LSEG codec needs a raw CBOR block and its compressed block.
/// This reservation is deliberately part of, rather than outside, the caller
/// supplied scratch budget.
const MIN_CODEC_BYTES: usize = 2 * 1024 * 1024;
const PAYLOAD_SLOTS: usize = 16;
const MAX_BLOCK_ENTRIES: usize = 256;
const RUN_RECORD_OVERHEAD: usize = 2 * std::mem::size_of::<String>() + 32;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RequiredTextRowWorkspace {
    pub(crate) required_bytes: usize,
}

impl std::fmt::Display for RequiredTextRowWorkspace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "text row staging requires {} scratch bytes",
            self.required_bytes
        )
    }
}
impl std::error::Error for RequiredTextRowWorkspace {}

#[derive(Clone, Copy, Debug)]
pub(crate) struct TextRowStageOptions {
    /// Total writer-owned RAM.  It excludes the borrowed token supplied by the
    /// caller, but includes every cloned run term and codec buffer.
    pub scratch_bytes: usize,
}

impl TextRowStageOptions {
    pub(crate) fn minimum_scratch_bytes() -> usize {
        MIN_CODEC_BYTES + PAYLOAD_SLOTS * RUN_RECORD_OVERHEAD
    }

    fn run_bytes(self) -> Result<usize> {
        self.scratch_bytes
            .checked_sub(MIN_CODEC_BYTES)
            .map(|bytes| bytes / PAYLOAD_SLOTS)
            .filter(|bytes| *bytes >= RUN_RECORD_OVERHEAD)
            .ok_or_else(|| {
                anyhow!(
                    "text row staging needs at least {} scratch bytes for the current LSEG codec",
                    Self::minimum_scratch_bytes()
                )
            })
    }
}

/// Feed borrowed normalized tokens to `emit`.  The stream must preserve token
/// repetitions; this writer derives each term frequency while it sorts runs.
pub(crate) type TextTokenStream<'a> =
    dyn FnOnce(&mut dyn FnMut(&str) -> Result<()>) -> Result<()> + 'a;

/// Build a current-format one-row Text segment at `path`.
///
/// `document_len` is the tokenizer's checked total token count.  An empty
/// token stream still writes an explicitly present row with document length
/// zero.  `scratch_dir` must be on the same filesystem as `path`, because the
/// final artifact is atomically renamed into place.
pub(crate) fn stage_text_row(
    path: &Path,
    applied_seq: u64,
    document_len: u32,
    scratch_dir: &Path,
    options: TextRowStageOptions,
    stream: Box<TextTokenStream<'_>>,
) -> Result<()> {
    let run_bytes = options.run_bytes()?;
    let mut workspace = RowWorkspace::new(scratch_dir, run_bytes)?;
    let mut terms = Vec::<String>::new();
    let mut used = 0usize;
    let mut feed_error = None;
    let mut seen = 0u64;
    let mut emit = |term: &str| -> Result<()> {
        seen = seen
            .checked_add(1)
            .ok_or_else(|| anyhow!("text token count overflow"))?;
        if term.is_empty() {
            bail!("normalized Text stream emitted an empty token");
        }
        let record = term
            .len()
            .checked_add(RUN_RECORD_OVERHEAD)
            .ok_or_else(|| anyhow!("text term length overflow"))?;
        if record > run_bytes {
            // Reprice the bounded workspace before any output. The same
            // on-disk format handles the term after the caller reserves it.
            let required_bytes = record
                .checked_mul(PAYLOAD_SLOTS)
                .and_then(|bytes| MIN_CODEC_BYTES.checked_add(bytes))
                .ok_or_else(|| anyhow!("required text workspace overflows usize"))?;
            return Err(RequiredTextRowWorkspace { required_bytes }.into());
        }
        if used
            .checked_add(record)
            .ok_or_else(|| anyhow!("text run byte count overflow"))?
            > run_bytes
        {
            flush_run(&mut workspace, &mut terms, &mut used)?;
        }
        terms.push(term.to_owned());
        used = used
            .checked_add(record)
            .ok_or_else(|| anyhow!("text run byte count overflow"))?;
        Ok(())
    };
    if let Err(error) = stream(&mut emit) {
        feed_error = Some(error);
    }
    drop(emit);
    if let Some(error) = feed_error {
        return Err(error);
    }
    if seen != u64::from(document_len) {
        bail!("text row document length mismatch: declared {document_len}, streamed {seen}");
    }
    flush_run(&mut workspace, &mut terms, &mut used)?;
    let run = workspace.finish_runs()?;
    drop(terms);
    write_lseg_from_run(
        path,
        applied_seq,
        document_len,
        run.as_deref(),
        options,
        workspace.dir(),
    )?;
    // The final LSEG is outside the private workspace.  Drop removes every
    // intermediate run after the atomic final rename succeeds.
    Ok(())
}

struct RowWorkspace {
    dir: PathBuf,
    levels: Vec<Option<PathBuf>>,
    armed: bool,
    max_term: usize,
}

impl RowWorkspace {
    fn new(parent: &Path, max_term: usize) -> Result<Self> {
        fs::create_dir_all(parent)
            .with_context(|| format!("create text row staging directory {}", parent.display()))?;
        let nonce = ROW_STAGE_NONCE.fetch_add(1, AtomicOrdering::Relaxed);
        let dir = parent.join(format!(".text-row-stage-{}-{nonce}", std::process::id()));
        fs::create_dir(&dir)
            .with_context(|| format!("create text row workspace {}", dir.display()))?;
        Ok(Self {
            dir,
            levels: Vec::new(),
            armed: true,
            max_term,
        })
    }

    fn dir(&self) -> &Path {
        &self.dir
    }

    fn push_run(&mut self, mut run: PathBuf) -> Result<()> {
        let mut level = 0usize;
        loop {
            if self.levels.len() == level {
                self.levels.push(Some(run));
                return Ok(());
            }
            match self.levels[level].take() {
                None => {
                    self.levels[level] = Some(run);
                    return Ok(());
                }
                Some(left) => {
                    // Merge immediately like a binary carry.  This retains
                    // O(log runs) paths and opens exactly two inputs.
                    run = merge_runs(self.dir(), &left, &run, self.max_term)?;
                    level += 1;
                }
            }
        }
    }

    fn finish_runs(&mut self) -> Result<Option<PathBuf>> {
        let mut runs: Vec<PathBuf> = self.levels.iter_mut().filter_map(Option::take).collect();
        if runs.is_empty() {
            return Ok(None);
        }
        // Pairwise merging keeps at most two input files open.  The vector
        // stores only O(log runs) paths after normal ingestion; final folding
        // is intentionally sequential and does not open all paths.
        let mut result = runs.remove(0);
        for next in runs {
            result = merge_runs(self.dir(), &result, &next, self.max_term)?;
        }
        Ok(Some(result))
    }
}

impl Drop for RowWorkspace {
    fn drop(&mut self) {
        if self.armed {
            let _ = fs::remove_dir_all(&self.dir);
        }
    }
}

fn fresh_path(dir: &Path, kind: &str) -> PathBuf {
    let nonce = ROW_STAGE_NONCE.fetch_add(1, AtomicOrdering::Relaxed);
    dir.join(format!("{kind}-{}-{nonce}.run", std::process::id()))
}

fn flush_run(
    workspace: &mut RowWorkspace,
    terms: &mut Vec<String>,
    used: &mut usize,
) -> Result<()> {
    if terms.is_empty() {
        return Ok(());
    }
    terms.sort_unstable();
    let path = fresh_path(workspace.dir(), "run");
    let mut out = BufWriter::new(
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .with_context(|| format!("create text term run {}", path.display()))?,
    );
    let mut index = 0usize;
    while index < terms.len() {
        let first = index;
        index += 1;
        while index < terms.len() && terms[index] == terms[first] {
            index += 1;
        }
        let count: u32 = (index - first)
            .try_into()
            .context("text term frequency exceeds u32")?;
        write_run_record(&mut out, terms[first].as_bytes(), count)?;
    }
    out.flush()
        .with_context(|| format!("flush text term run {}", path.display()))?;
    out.get_ref()
        .sync_all()
        .with_context(|| format!("fsync text term run {}", path.display()))?;
    terms.clear();
    *used = 0;
    workspace.push_run(path)?;
    Ok(())
}

fn write_run_record(out: &mut impl Write, term: &[u8], count: u32) -> Result<()> {
    let length: u32 = term
        .len()
        .try_into()
        .context("text term exceeds u32 record capacity")?;
    let length = length.to_le_bytes();
    let count = count.to_le_bytes();
    let mut crc = crc32fast::Hasher::new();
    crc.update(&length);
    crc.update(term);
    crc.update(&count);
    out.write_all(&length)?;
    out.write_all(term)?;
    out.write_all(&count)?;
    out.write_all(&crc.finalize().to_le_bytes())?;
    Ok(())
}

struct RunReader {
    input: BufReader<File>,
    current: Option<(Vec<u8>, u32)>,
    max_term: usize,
}
impl RunReader {
    fn open(path: &Path, max_term: usize) -> Result<Self> {
        let mut reader = Self {
            input: BufReader::new(File::open(path)?),
            current: None,
            max_term,
        };
        reader.advance()?;
        Ok(reader)
    }
    fn advance(&mut self) -> Result<()> {
        // Release the old buffer before allocating the next. A clean EOF is
        // valid only before the first length byte; a short prefix is corruption.
        self.current = None;
        let mut length = [0u8; 4];
        match self.input.read(&mut length[..1]) {
            Ok(0) => {
                self.current = None;
                return Ok(());
            }
            Ok(1) => {}
            Ok(_) => unreachable!("one-byte run prefix read"),
            Err(error) => return Err(error.into()),
        }
        self.input
            .read_exact(&mut length[1..])
            .context("truncated text term run length prefix")?;
        let length = usize::try_from(u32::from_le_bytes(length))
            .context("run term length exceeds platform")?;
        anyhow::ensure!(
            length <= self.max_term,
            "text run length exceeds reserved term buffer"
        );
        let mut term = vec![0; length];
        self.input.read_exact(&mut term)?;
        let mut count = [0u8; 4];
        self.input
            .read_exact(&mut count)
            .context("truncated text term run count")?;
        let mut checksum = [0u8; 4];
        self.input
            .read_exact(&mut checksum)
            .context("truncated text term run checksum")?;
        let mut crc = crc32fast::Hasher::new();
        crc.update(&(length as u32).to_le_bytes());
        crc.update(&term);
        crc.update(&count);
        anyhow::ensure!(
            crc.finalize() == u32::from_le_bytes(checksum),
            "text term run checksum mismatch"
        );
        let count = u32::from_le_bytes(count);
        anyhow::ensure!(
            !term.is_empty() && count != 0 && std::str::from_utf8(&term).is_ok(),
            "invalid normalized text term run record"
        );
        self.current = Some((term, count));
        Ok(())
    }
}

fn merge_runs(dir: &Path, left: &Path, right: &Path, max_term: usize) -> Result<PathBuf> {
    let path = fresh_path(dir, "merge");
    let result = (|| {
        let mut a = RunReader::open(left, max_term)?;
        let mut b = RunReader::open(right, max_term)?;
        let mut out = BufWriter::new(
            OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)?,
        );
        loop {
            let pick = match (&a.current, &b.current) {
                (None, None) => break,
                (Some(_), None) => Ordering::Less,
                (None, Some(_)) => Ordering::Greater,
                (Some((a_term, _)), Some((b_term, _))) => a_term.cmp(b_term),
            };
            match pick {
                Ordering::Less => {
                    let (term, count) = a.current.as_ref().unwrap();
                    write_run_record(&mut out, term, *count)?;
                    a.advance()?;
                }
                Ordering::Greater => {
                    let (term, count) = b.current.as_ref().unwrap();
                    write_run_record(&mut out, term, *count)?;
                    b.advance()?;
                }
                Ordering::Equal => {
                    let (term, a_count) = a.current.as_ref().unwrap();
                    let b_count = b.current.as_ref().unwrap().1;
                    let count = a_count
                        .checked_add(b_count)
                        .ok_or_else(|| anyhow!("text term frequency exceeds u32"))?;
                    write_run_record(&mut out, term, count)?;
                    a.advance()?;
                    b.advance()?;
                }
            }
        }
        out.flush()?;
        out.get_ref().sync_all()?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&path);
        return result.map(|_| path);
    }
    fs::remove_file(left).with_context(|| format!("remove merged run {}", left.display()))?;
    fs::remove_file(right).with_context(|| format!("remove merged run {}", right.display()))?;
    Ok(path)
}

// Skip metadata is a disk spool, independent of dictionary cardinality.
struct RowVarWriter {
    pending: Vec<VarEntry>,
    pending_bytes: usize,
    prev: Vec<u8>,
    block_first: u32,
    next_id: u32,
    metadata: BufWriter<File>,
    metadata_path: PathBuf,
    blocks: u64,
}
impl RowVarWriter {
    fn new(workspace: &Path) -> Result<Self> {
        let metadata_path = fresh_path(workspace, "block-meta");
        let metadata = BufWriter::new(
            OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&metadata_path)?,
        );
        Ok(Self {
            pending: Vec::with_capacity(MAX_BLOCK_ENTRIES),
            pending_bytes: 0,
            prev: Vec::new(),
            block_first: 0,
            next_id: 0,
            metadata,
            metadata_path,
            blocks: 0,
        })
    }
    fn push(&mut self, entry: &[u8], out: &mut BufWriter<File>, at: &mut u64) -> Result<()> {
        if !self.pending.is_empty()
            && (self.pending.len() == MAX_BLOCK_ENTRIES
                || self
                    .pending_bytes
                    .checked_add(entry.len())
                    .is_none_or(|n| n > VAR_BLOCK_BYTES))
        {
            self.flush(out, at)?;
        }
        let shared = u32::try_from(shared_prefix(&self.prev, entry))?;
        let suffix = entry[shared as usize..].to_vec();
        self.pending_bytes = self
            .pending_bytes
            .checked_add(suffix.len())
            .ok_or_else(|| anyhow!("text var block size overflow"))?;
        self.pending.push(VarEntry { shared, suffix });
        self.prev.clear();
        self.prev.extend_from_slice(entry);
        self.next_id = self
            .next_id
            .checked_add(1)
            .ok_or_else(|| anyhow!("text dictionary exceeds u32 ordinal capacity"))?;
        if self.pending_bytes >= VAR_BLOCK_BYTES {
            self.flush(out, at)?;
        }
        Ok(())
    }
    fn flush(&mut self, out: &mut BufWriter<File>, at: &mut u64) -> Result<()> {
        if self.pending.is_empty() {
            return Ok(());
        }
        // A suffix byte takes at most two CBOR bytes. Per-entry framing,
        // keys and integer sizes fit in another 64 bytes.
        let capacity = self
            .pending_bytes
            .checked_mul(2)
            .and_then(|n| n.checked_add(self.pending.len() * 64 + 64))
            .ok_or_else(|| anyhow!("text block encoded size overflow"))?;
        let mut raw = BoundedBytes(Vec::with_capacity(capacity));
        #[derive(serde::Serialize)]
        struct Body<'a> {
            entries: &'a [VarEntry],
        }
        ciborium::into_writer(
            &Body {
                entries: &self.pending,
            },
            &mut raw,
        )
        .map_err(|e| anyhow!("encode text staged var block: {e}"))?;
        let compressed = lz4_flex::compress_prepend_size(&raw.0);
        let length = u32::try_from(compressed.len())?;
        let offset = *at;
        counted(out, at, &length.to_le_bytes())?;
        counted(out, at, &compressed)?;
        self.metadata.write_all(&self.block_first.to_le_bytes())?;
        self.metadata
            .write_all(&(self.pending.len() as u32).to_le_bytes())?;
        self.metadata.write_all(&offset.to_le_bytes())?;
        self.metadata.write_all(&length.to_le_bytes())?;
        self.blocks = self
            .blocks
            .checked_add(1)
            .ok_or_else(|| anyhow!("text index count overflow"))?;
        self.pending.clear();
        self.pending_bytes = 0;
        self.prev.clear();
        self.block_first = self.next_id;
        Ok(())
    }
    fn finish(
        mut self,
        out: &mut BufWriter<File>,
        at: &mut u64,
        offset: u64,
        workspace: &Path,
    ) -> Result<(DiskBytes, u64, u64, u64)> {
        self.flush(out, at)?;
        self.metadata.flush()?;
        drop(self.metadata);
        let skip_path = fresh_path(workspace, "skip-index");
        let mut skip = BufWriter::new(
            OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&skip_path)?,
        );
        let source = MetaSpool {
            path: self.metadata_path,
            count: self.blocks,
        };
        #[derive(serde::Serialize)]
        struct Index<'a> {
            blocks: &'a MetaSpool,
        }
        ciborium::into_writer(&Index { blocks: &source }, &mut skip)
            .map_err(|e| anyhow!("encode text staged skip index: {e}"))?;
        skip.flush()?;
        let len = skip.get_ref().metadata()?.len();
        Ok((
            DiskBytes {
                path: skip_path,
                len,
            },
            offset,
            *at - offset,
            self.next_id as u64,
        ))
    }
}

struct BoundedBytes(Vec<u8>);
impl Write for BoundedBytes {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if bytes.len() > self.0.capacity() - self.0.len() {
            return Err(std::io::Error::other(
                "text codec exceeded reserved output capacity",
            ));
        }
        self.0.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

struct MetaSpool {
    path: PathBuf,
    count: u64,
}
impl serde::Serialize for MetaSpool {
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        use serde::{ser::Error, ser::SerializeSeq};
        let mut input = BufReader::new(File::open(&self.path).map_err(S::Error::custom)?);
        let count = usize::try_from(self.count).map_err(S::Error::custom)?;
        let mut seq = serializer.serialize_seq(Some(count))?;
        for _ in 0..count {
            let mut bytes = [0u8; 20];
            input.read_exact(&mut bytes).map_err(S::Error::custom)?;
            seq.serialize_element(&VarBlockMeta {
                first_entry: u32::from_le_bytes(bytes[0..4].try_into().unwrap()),
                entry_count: u32::from_le_bytes(bytes[4..8].try_into().unwrap()),
                offset: u64::from_le_bytes(bytes[8..16].try_into().unwrap()),
                length: u32::from_le_bytes(bytes[16..20].try_into().unwrap()),
            })?;
        }
        let mut extra = [0u8; 1];
        if input.read(&mut extra).map_err(S::Error::custom)? != 0 {
            return Err(S::Error::custom(
                "text block metadata spool has trailing bytes",
            ));
        }
        seq.end()
    }
}

struct DiskBytes {
    path: PathBuf,
    len: u64,
}
impl serde::Serialize for DiskBytes {
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        use serde::{ser::Error, ser::SerializeSeq};
        let mut input = BufReader::new(File::open(&self.path).map_err(S::Error::custom)?);
        let len = usize::try_from(self.len).map_err(S::Error::custom)?;
        // Vec<u8> in ColumnRef is an integer array, not a CBOR byte string.
        let mut seq = serializer.serialize_seq(Some(len))?;
        let mut buffer = [0u8; 4096];
        let mut remaining = len;
        while remaining != 0 {
            let n = remaining.min(buffer.len());
            input
                .read_exact(&mut buffer[..n])
                .map_err(S::Error::custom)?;
            for byte in &buffer[..n] {
                seq.serialize_element(byte)?;
            }
            remaining -= n;
        }
        seq.end()
    }
}

struct DiskColumn {
    column: ColumnRef,
    skip: Option<DiskBytes>,
}
impl serde::Serialize for DiskColumn {
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut s =
            serializer.serialize_struct("ColumnRef", if self.skip.is_some() { 8 } else { 7 })?;
        s.serialize_field("name", &self.column.name)?;
        s.serialize_field("role", &self.column.role)?;
        s.serialize_field("byte_offset", &self.column.byte_offset)?;
        s.serialize_field("byte_len", &self.column.byte_len)?;
        s.serialize_field("elem_count", &self.column.elem_count)?;
        s.serialize_field("width", &self.column.width)?;
        s.serialize_field("codec", &self.column.codec)?;
        if let Some(skip) = &self.skip {
            s.serialize_field("skip_index", skip)?;
        }
        s.end()
    }
}
struct DirectoryOutput<'a> {
    out: &'a mut BufWriter<File>,
    len: u64,
    crc: crc32fast::Hasher,
}
impl Write for DirectoryOutput<'_> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.out.write_all(bytes)?;
        self.len = self
            .len
            .checked_add(bytes.len() as u64)
            .ok_or_else(|| std::io::Error::other("text directory length overflow"))?;
        self.crc.update(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.out.flush()
    }
}

fn counted(out: &mut BufWriter<File>, at: &mut u64, bytes: &[u8]) -> Result<()> {
    out.write_all(bytes)?;
    *at = at
        .checked_add(bytes.len() as u64)
        .ok_or_else(|| anyhow!("segment byte offset overflow"))?;
    Ok(())
}
fn zeroes(out: &mut BufWriter<File>, at: &mut u64, mut n: usize) -> Result<()> {
    const Z: [u8; 4096] = [0; 4096];
    while n > 0 {
        let take = n.min(Z.len());
        counted(out, at, &Z[..take])?;
        n -= take;
    }
    Ok(())
}
fn align(out: &mut BufWriter<File>, at: &mut u64) -> Result<()> {
    let size = usize::try_from(*at).context("segment exceeds platform size")?;
    zeroes(out, at, page_align(size) - size)
}

fn write_lseg_from_run(
    path: &Path,
    seq: u64,
    document_len: u32,
    run: Option<&Path>,
    options: TextRowStageOptions,
    workspace: &Path,
) -> Result<()> {
    let max_term = options.run_bytes()?;
    let parent = path
        .parent()
        .ok_or_else(|| anyhow!("text row path has no parent: {}", path.display()))?;
    fs::create_dir_all(parent)?;
    let temp = parent.join(format!(
        ".{}-{}.tmp",
        path.file_name().and_then(|n| n.to_str()).unwrap_or("row"),
        ROW_STAGE_NONCE.fetch_add(1, AtomicOrdering::Relaxed)
    ));
    let result = (|| {
        let mut out = BufWriter::new(
            OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&temp)?,
        );
        let mut at = 0u64;
        counted(
            &mut out,
            &mut at,
            &header_block(seq, 1, 1, document_len as u64),
        )?;
        let doclen_off = at;
        counted(&mut out, &mut at, &document_len.to_le_bytes())?;
        let doclen_len = at - doclen_off;
        let padding = ((8 - at % 8) % 8) as usize;
        zeroes(&mut out, &mut at, padding)?;
        let present_off = at;
        counted(&mut out, &mut at, &1u64.to_le_bytes())?;
        let present_len = at - present_off;
        align(&mut out, &mut at)?;
        let mut input = match run {
            Some(path) => Some(RunReader::open(path, max_term)?),
            None => None,
        };
        let dict_off = at;
        let mut dict = RowVarWriter::new(workspace)?;
        while let Some(reader) = input.as_mut() {
            let Some((term, _)) = reader.current.as_ref() else {
                break;
            };
            if term.len() > max_term {
                bail!("single text term of {} bytes exceeds staging scratch payload {}; use dedicated large-term record encoding",term.len(),max_term);
            }
            dict.push(term, &mut out, &mut at)?;
            reader.advance()?;
        }
        let (dict_skip, dict_off, dict_len, dict_count) =
            dict.finish(&mut out, &mut at, dict_off, workspace)?;
        let mut input = match run {
            Some(path) => Some(RunReader::open(path, max_term)?),
            None => None,
        };
        let postings_off = at;
        let mut postings = RowVarWriter::new(workspace)?;
        while let Some(reader) = input.as_mut() {
            let Some((_, count)) = reader.current.as_ref() else {
                break;
            };
            let mut blob = Vec::with_capacity(12);
            write_varint(&mut blob, 1);
            write_varint(&mut blob, 0);
            write_varint(&mut blob, *count as u64);
            postings.push(&blob, &mut out, &mut at)?;
            reader.advance()?;
        }
        let (postings_skip, postings_off, postings_len, postings_count) =
            postings.finish(&mut out, &mut at, postings_off, workspace)?;
        if dict_count != postings_count {
            bail!("staged text dictionary/posting ordinal mismatch");
        }
        let dir = vec![
            ColumnRef {
                name: "text_doclen".to_owned(),
                role: ROLE_TEXT_DOCLEN,
                byte_offset: doclen_off,
                byte_len: doclen_len,
                elem_count: 1,
                width: 4,
                codec: CODEC_FIXED,
                skip_index: Vec::new(),
            },
            present_column_ref(present_off, present_len, 1),
            ColumnRef {
                name: "dict".to_owned(),
                role: ROLE_DICT,
                byte_offset: dict_off,
                byte_len: dict_len,
                elem_count: dict_count,
                width: 0,
                codec: CODEC_LZ4_VAR,
                skip_index: Vec::new(),
            },
            ColumnRef {
                name: "text_postings".to_owned(),
                role: ROLE_TEXT_POSTINGS,
                byte_offset: postings_off,
                byte_len: postings_len,
                elem_count: postings_count,
                width: 0,
                codec: CODEC_LZ4_VAR,
                skip_index: Vec::new(),
            },
        ];
        let mut skips = [None, None, Some(dict_skip), Some(postings_skip)].into_iter();
        let dir: Vec<_> = dir
            .into_iter()
            .map(|column| DiskColumn {
                column,
                skip: skips.next().unwrap(),
            })
            .collect();
        let dir_offset = at;
        let mut output = DirectoryOutput {
            out: &mut out,
            len: 0,
            crc: crc32fast::Hasher::new(),
        };
        ciborium::into_writer(&dir, &mut output)
            .map_err(|e| anyhow!("encode staged text directory: {e}"))?;
        let footer = Footer {
            dir_offset,
            dir_len: output.len,
            crc32: output.crc.finalize(),
            magic2: MAGIC2,
        };
        at = at
            .checked_add(footer.dir_len)
            .ok_or_else(|| anyhow!("text directory offset overflow"))?;
        counted(&mut out, &mut at, &footer.to_bytes())?;
        out.flush()?;
        out.get_ref().sync_all()?;
        drop(out);
        fs::rename(&temp, path)?;
        // The name is part of the prepared artifact's durable ownership. A
        // file sync alone does not preserve a rename across power loss.
        File::open(
            path.parent()
                .ok_or_else(|| anyhow!("text row has no parent"))?,
        )?
        .sync_all()?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temp);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stage(path: &Path, tokens: &[&str], len: u32, budget: usize) -> Result<()> {
        stage_text_row(
            path,
            41,
            len,
            path.parent().unwrap(),
            TextRowStageOptions {
                scratch_bytes: budget,
            },
            Box::new(|emit| {
                for token in tokens {
                    emit(token)?;
                }
                Ok(())
            }),
        )
    }

    #[test]
    fn row_stage_round_trips_sorted_terms_and_duplicate_frequency() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("row.lseg");
        stage(
            &path,
            &["zebra", "ant", "zebra", "bee", "ant"],
            5,
            TextRowStageOptions::minimum_scratch_bytes() + 128,
        )
        .unwrap();
        let reader = SegmentReader::open(&path).unwrap();
        assert_eq!(reader.applied_seq(), 41);
        assert!(reader.text_is_present(0));
        assert_eq!(reader.text_doc_len(0), 5);
        assert_eq!(reader.text_postings("ant"), Some((vec![0], vec![2])));
        assert_eq!(reader.text_postings("bee"), Some((vec![0], vec![1])));
        assert_eq!(reader.text_postings("zebra"), Some((vec![0], vec![2])));
    }

    #[test]
    fn row_stage_merges_duplicate_terms_across_runs() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("row.lseg");
        stage(
            &path,
            &["same", "b", "same", "a", "same", "b"],
            6,
            TextRowStageOptions::minimum_scratch_bytes() + RUN_RECORD_OVERHEAD + 4,
        )
        .unwrap();
        let reader = SegmentReader::open(&path).unwrap();
        assert_eq!(reader.text_postings("same"), Some((vec![0], vec![3])));
        assert_eq!(reader.text_postings("b"), Some((vec![0], vec![2])));
    }

    #[test]
    fn row_stage_keeps_unicode_and_empty_present_row() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("row.lseg");
        stage(
            &path,
            &["台北", "ß"],
            2,
            TextRowStageOptions::minimum_scratch_bytes() + 128,
        )
        .unwrap();
        let reader = SegmentReader::open(&path).unwrap();
        assert_eq!(reader.text_postings("台北"), Some((vec![0], vec![1])));
        let empty = temp.path().join("empty.lseg");
        stage(
            &empty,
            &[],
            0,
            TextRowStageOptions::minimum_scratch_bytes() + 128,
        )
        .unwrap();
        let reader = SegmentReader::open(&empty).unwrap();
        assert!(reader.text_is_present(0));
        assert_eq!(reader.text_doc_len(0), 0);
    }

    #[test]
    fn row_stage_reprices_workspace_and_accepts_one_term_larger_than_a_var_block() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("row.lseg");
        let budget = TextRowStageOptions::minimum_scratch_bytes() + RUN_RECORD_OVERHEAD;
        let token = "x".repeat(VAR_BLOCK_BYTES + 1);
        let error = stage(&path, &[token.as_str()], 1, budget).unwrap_err();
        let required = error
            .downcast_ref::<RequiredTextRowWorkspace>()
            .unwrap()
            .required_bytes;
        assert!(required > budget);
        assert!(!path.exists());
        assert!(fs::read_dir(temp.path()).unwrap().next().is_none());
        stage(&path, &[token.as_str()], 1, required).unwrap();
        let reader = SegmentReader::open(&path).unwrap();
        assert_eq!(reader.text_postings(&token), Some((vec![0], vec![1])));
    }

    #[test]
    fn row_stage_rejects_wrong_declared_document_length() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("row.lseg");
        let error = stage(
            &path,
            &["one", "two"],
            1,
            TextRowStageOptions::minimum_scratch_bytes() + 128,
        )
        .unwrap_err();
        assert!(error.to_string().contains("document length mismatch"));
        assert!(!path.exists());
    }

    #[test]
    fn run_reader_rejects_partial_length_prefix() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("truncated.run");
        fs::write(&path, [3u8, 0]).unwrap();
        let error = match RunReader::open(&path, 1024) {
            Ok(_) => panic!("partial prefix accepted"),
            Err(error) => error,
        };
        assert!(error
            .to_string()
            .contains("truncated text term run length prefix"));
    }

    #[test]
    fn row_stage_rejects_too_small_budget_and_cleans_workspace() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("row.lseg");
        let error = stage(
            &path,
            &["token"],
            1,
            TextRowStageOptions::minimum_scratch_bytes() - 1,
        )
        .unwrap_err();
        assert!(error.to_string().contains("needs at least"));
        assert!(!path.exists());
        assert!(fs::read_dir(temp.path()).unwrap().next().is_none());
    }

    #[test]
    fn row_stage_removes_output_temp_after_rename_error() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("occupied");
        fs::create_dir(&path).unwrap();
        let error = stage(
            &path,
            &["term"],
            1,
            TextRowStageOptions::minimum_scratch_bytes() + 64,
        )
        .unwrap_err();
        assert!(
            error.to_string().contains("rename") || error.to_string().contains("Is a directory")
        );
        assert!(fs::read_dir(&path).unwrap().next().is_none());
        assert_eq!(fs::read_dir(temp.path()).unwrap().count(), 1);
    }

    #[test]
    fn row_stage_removes_temporary_files_after_stream_error() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("row.lseg");
        let result = stage_text_row(
            &path,
            1,
            1,
            temp.path(),
            TextRowStageOptions {
                scratch_bytes: TextRowStageOptions::minimum_scratch_bytes() + 64,
            },
            Box::new(|emit| {
                emit("ok")?;
                bail!("injected stream failure")
            }),
        );
        assert!(result.unwrap_err().to_string().contains("injected"));
        assert!(!path.exists());
        assert!(fs::read_dir(temp.path()).unwrap().next().is_none());
    }
    #[test]
    fn disk_spooled_directory_preserves_ordinals_across_many_var_blocks() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("many.lseg");
        let tokens: Vec<_> = (0..1300).map(|i| format!("term-{i:04}")).collect();
        stage_text_row(
            &path,
            17,
            tokens.len() as u32,
            temp.path(),
            TextRowStageOptions {
                scratch_bytes: MIN_CODEC_BYTES + PAYLOAD_SLOTS * 200_000,
            },
            Box::new(|emit| {
                for token in tokens.iter().rev() {
                    emit(token)?;
                }
                Ok(())
            }),
        )
        .unwrap();
        let reader = SegmentReader::open(&path).unwrap();
        assert_eq!(reader.text_doc_len(0), 1300);
        for ordinal in [0, 255, 256, 511, 512, 1024, 1299] {
            assert_eq!(
                reader.text_postings(&tokens[ordinal]),
                Some((vec![0], vec![1]))
            );
        }
        assert_eq!(
            fs::read_dir(temp.path()).unwrap().count(),
            1,
            "skip metadata and all run files are scoped scratch"
        );
    }

    #[test]
    fn row_stage_run_rejects_payload_and_frequency_bit_rot() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("corrupt.run");
        let mut original = Vec::new();
        write_run_record(&mut original, b"term", 3).unwrap();
        // Each mutation remains structurally valid: one changes an ASCII term
        // and the other changes a positive TF. Size checks alone cannot catch it.
        for (offset, context) in [(4, "term payload"), (8, "term frequency")] {
            let mut corrupt = original.clone();
            corrupt[offset] ^= 1;
            fs::write(&path, corrupt).unwrap();
            assert!(
                RunReader::open(&path, 128).is_err(),
                "run readback must reject bit rot in {context} before publishing a Text segment"
            );
        }
    }

    #[test]
    fn row_stage_run_rejects_empty_invalid_utf8_and_zero_frequency() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("invalid.run");
        for (term, count) in [(b"".as_slice(), 1), (&[0xff], 1), (b"valid", 0)] {
            let mut encoded = Vec::new();
            write_run_record(&mut encoded, term, count).unwrap();
            fs::write(&path, encoded).unwrap();
            assert!(
                RunReader::open(&path, 128).is_err(),
                "invalid normalized run record must fail before segment publication"
            );
        }
    }

    #[test]
    fn codec_output_cannot_reallocate_past_its_reserved_capacity() {
        let mut bytes = BoundedBytes(Vec::with_capacity(3));
        let capacity = bytes.0.capacity();
        bytes.write_all(&[1, 2, 3]).unwrap();
        assert!(bytes.write_all(&[4]).is_err());
        assert_eq!(bytes.0.capacity(), capacity);
        assert_eq!(bytes.0, vec![1, 2, 3]);
    }
}

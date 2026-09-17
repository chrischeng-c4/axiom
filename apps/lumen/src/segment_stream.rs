//! Streaming Text segment seal support for a composed view.
//!
//! This is included below `segment.rs`, so it deliberately uses that module's
//! private on-disk codec types.  It never collects the text dictionary or all
//! postings: each cursor pass owns at most one term and its one decoded posting.

use super::*;
use crate::composed_segment::ComposedSegmentReader;
use std::borrow::Cow;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static STREAM_TEMP_NONCE: AtomicU64 = AtomicU64::new(0);

/// A file-backed version of [`VarColumnWriter`].  Its current block is the
/// only materialized var-column data.  The skip index is intentionally kept in
/// memory because it is small metadata needed by the CBOR directory.
struct StreamingVarWriter {
    pending: Vec<VarEntry>,
    pending_bytes: usize,
    full_pending_bytes: usize,
    bound_decoded_block: bool,
    prev: Vec<u8>,
    block_first: u32,
    next_id: u32,
    index: Vec<VarBlockMeta>,
}

impl StreamingVarWriter {
    fn new() -> Self {
        Self {
            pending: Vec::new(),
            pending_bytes: 0,
            full_pending_bytes: 0,
            bound_decoded_block: false,
            prev: Vec::new(),
            block_first: 0,
            next_id: 0,
            index: Vec::new(),
        }
    }

    /// Keep reconstructed prefix-delta strings within one bounded block.
    /// Prefix compression still applies, but shared prefixes cannot turn a tiny
    /// compressed block into a collection-sized decoded allocation.
    fn bounded_projection() -> Self {
        Self {
            bound_decoded_block: true,
            ..Self::new()
        }
    }

    fn push(&mut self, entry: &[u8], out: &mut BufWriter<File>, at: &mut u64) -> Result<()> {
        self.full_pending_bytes = self
            .full_pending_bytes
            .checked_add(entry.len())
            .and_then(|n| n.checked_add(4))
            .ok_or_else(|| anyhow!("stream decoded block size overflow"))?;
        let shared = shared_prefix(&self.prev, entry) as u32;
        let suffix = entry[shared as usize..].to_vec();
        self.pending_bytes = self
            .pending_bytes
            .checked_add(4 + suffix.len())
            .ok_or_else(|| anyhow!("stream var block size overflow"))?;
        self.pending.push(VarEntry { shared, suffix });
        self.prev.clear();
        self.prev.extend_from_slice(entry);
        self.next_id = self
            .next_id
            .checked_add(1)
            .ok_or_else(|| anyhow!("text dictionary exceeds u32 ordinal capacity"))?;
        if self.pending_bytes >= VAR_BLOCK_BYTES
            || (self.bound_decoded_block && self.full_pending_bytes >= VAR_BLOCK_BYTES)
        {
            self.flush(out, at)?;
        }
        Ok(())
    }

    fn flush(&mut self, out: &mut BufWriter<File>, at: &mut u64) -> Result<()> {
        if self.pending.is_empty() {
            return Ok(());
        }
        let body = VarBlockBody {
            entries: std::mem::take(&mut self.pending),
        };
        let mut raw = Vec::new();
        ciborium::into_writer(&body, &mut raw)
            .map_err(|error| anyhow!("encode streamed var block: {error}"))?;
        let compressed = lz4_flex::compress_prepend_size(&raw);
        let length: u32 = compressed
            .len()
            .try_into()
            .context("streamed var block exceeds u32 compressed length")?;
        let offset = *at;
        write_counted(out, at, &length.to_le_bytes())?;
        write_counted(out, at, &compressed)?;
        self.index.push(VarBlockMeta {
            first_entry: self.block_first,
            entry_count: body.entries.len() as u32,
            offset,
            length,
        });
        self.pending_bytes = 0;
        self.full_pending_bytes = 0;
        self.prev.clear();
        self.block_first = self.next_id;
        Ok(())
    }

    fn finish(
        mut self,
        out: &mut BufWriter<File>,
        at: &mut u64,
        byte_offset: u64,
    ) -> Result<(Vec<u8>, u64, u64, u64)> {
        self.flush(out, at)?;
        let mut skip_index = Vec::new();
        ciborium::into_writer(&SparseVarIndex { blocks: self.index }, &mut skip_index)
            .map_err(|error| anyhow!("encode streamed var skip-index: {error}"))?;
        Ok((
            skip_index,
            byte_offset,
            *at - byte_offset,
            self.next_id as u64,
        ))
    }
}

fn write_counted(out: &mut BufWriter<File>, at: &mut u64, bytes: &[u8]) -> Result<()> {
    out.write_all(bytes)?;
    *at = at
        .checked_add(bytes.len() as u64)
        .ok_or_else(|| anyhow!("segment byte offset overflow"))?;
    Ok(())
}

fn write_zeroes(out: &mut BufWriter<File>, at: &mut u64, count: usize) -> Result<()> {
    const ZEROES: [u8; 4096] = [0; 4096];
    let mut left = count;
    while left != 0 {
        let n = left.min(ZEROES.len());
        write_counted(out, at, &ZEROES[..n])?;
        left -= n;
    }
    Ok(())
}

fn pad_to_page(out: &mut BufWriter<File>, at: &mut u64) -> Result<()> {
    let aligned = page_align(usize::try_from(*at).context("segment exceeds platform size")?);
    write_zeroes(out, at, aligned - *at as usize)
}

fn new_stream_temp(path: &Path) -> Result<(PathBuf, File)> {
    let dir = path
        .parent()
        .ok_or_else(|| anyhow!("segment path has no parent: {}", path.display()))?;
    std::fs::create_dir_all(dir)
        .with_context(|| format!("create segment dir {}", dir.display()))?;
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| anyhow!("segment path has no file name: {}", path.display()))?;
    for _ in 0..32 {
        let nonce = STREAM_TEMP_NONCE.fetch_add(1, Ordering::Relaxed);
        let temp = dir.join(format!(".{name}.stream-{}-{nonce}.tmp", std::process::id()));
        match OpenOptions::new().write(true).create_new(true).open(&temp) {
            Ok(file) => return Ok((temp, file)),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error).with_context(|| format!("create {}", temp.display())),
        }
    }
    bail!(
        "could not allocate unique stream temp for {}",
        path.display()
    )
}

/// A sorted Text projection that can be sealed without rebuilding its
/// dictionary or postings.  `terms` may allocate one returned term at a time;
/// it must not retain the full dictionary.
pub(crate) trait TextStreamView {
    fn n_docs(&self) -> u32;
    fn text_is_present(&self, id: u32) -> bool;
    fn text_doc_len(&self, id: u32) -> u32;
    fn terms<'a>(&'a self) -> Result<Box<dyn Iterator<Item = Result<Cow<'a, str>>> + 'a>>;
    fn text_postings(&self, term: &str) -> Result<Option<std::sync::Arc<(Vec<u32>, Vec<u32>)>>>;
}

struct ComposedTerms<'a> {
    cursor: crate::composed_segment::StringTermCursor<'a>,
}

impl<'a> Iterator for ComposedTerms<'a> {
    type Item = Result<Cow<'a, str>>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.cursor.next_cow() {
            Ok(Some(term)) => Some(Ok(term)),
            Ok(None) => None,
            Err(error) => Some(Err(error)),
        }
    }
}

impl TextStreamView for ComposedSegmentReader {
    fn n_docs(&self) -> u32 {
        ComposedSegmentReader::n_docs(self)
    }
    fn text_is_present(&self, id: u32) -> bool {
        ComposedSegmentReader::text_is_present(self, id)
    }
    fn text_doc_len(&self, id: u32) -> u32 {
        ComposedSegmentReader::text_doc_len(self, id)
    }
    fn terms<'a>(&'a self) -> Result<Box<dyn Iterator<Item = Result<Cow<'a, str>>> + 'a>> {
        Ok(Box::new(ComposedTerms {
            cursor: self.string_terms(false)?,
        }))
    }
    fn text_postings(&self, term: &str) -> Result<Option<std::sync::Arc<(Vec<u32>, Vec<u32>)>>> {
        Ok(self.text_postings_arc(term))
    }
}

/// Seal a sorted Text projection without expanding its whole dictionary or
/// postings collection. `view` IDs are dense in `[0, view.n_docs())`.
pub(crate) fn write_text_projection(
    path: &Path,
    seq: u64,
    view: &impl TextStreamView,
) -> Result<()> {
    let n_docs = view.n_docs();
    let (temp, file) = new_stream_temp(path)?;
    let result = (|| {
        let mut out = BufWriter::new(file);
        let mut at = 0u64;
        let mut doc_count = 0u64;
        let mut total_doc_len = 0u64;
        for id in 0..n_docs {
            if view.text_is_present(id) {
                doc_count = doc_count
                    .checked_add(1)
                    .ok_or_else(|| anyhow!("text doc count overflow"))?;
                total_doc_len = total_doc_len
                    .checked_add(view.text_doc_len(id) as u64)
                    .ok_or_else(|| anyhow!("text total doc length overflow"))?;
            }
        }
        write_counted(
            &mut out,
            &mut at,
            &header_block(seq, n_docs, doc_count, total_doc_len),
        )?;
        let doclen_off = at;
        for id in 0..n_docs {
            write_counted(&mut out, &mut at, &view.text_doc_len(id).to_le_bytes())?;
        }
        let doclen_len = at - doclen_off;
        let present_pad = (8 - (at % 8)) % 8;
        write_zeroes(&mut out, &mut at, present_pad as usize)?;
        let present_off = at;
        let mut words = 0u64;
        let mut word = 0u64;
        for id in 0..n_docs {
            if view.text_is_present(id) {
                word |= 1u64 << (id % 64);
            }
            if id % 64 == 63 {
                write_counted(&mut out, &mut at, &word.to_le_bytes())?;
                words += 1;
                word = 0;
            }
        }
        if n_docs % 64 != 0 {
            write_counted(&mut out, &mut at, &word.to_le_bytes())?;
            words += 1;
        }
        let present_len = at - present_off;
        pad_to_page(&mut out, &mut at)?;
        // Text terms may lend an arbitrarily large raw mmap slice. The raw
        // dictionary codec writes that slice directly and spools only u64
        // boundaries, instead of constructing a term-sized LZ4/CBOR block.
        let (dict_off, dict_len, dict_count, dict_offsets_off, dict_offsets_len) =
            write_raw_dictionary(
                path,
                &mut out,
                &mut at,
                |emit| {
                    for term in view.terms()? {
                        let term = term?;
                        if view.text_postings(term.as_ref())?.is_some() {
                            emit(term.as_ref())?;
                        }
                    }
                    Ok(())
                },
                None,
            )?;
        let postings_start = at;
        let mut postings = StreamingVarWriter::new();
        for term in view.terms()? {
            let term = term?;
            let Some(posting) = view.text_postings(term.as_ref())? else {
                continue;
            };
            let blob = encode_posting_block(&posting.0, &posting.1);
            postings.push(&blob, &mut out, &mut at)?;
        }
        let (postings_skip, postings_off, postings_len, postings_count) =
            postings.finish(&mut out, &mut at, postings_start)?;
        if dict_count != postings_count {
            bail!("streamed text dictionary/posting ordinal mismatch: {dict_count} != {postings_count}");
        }
        let dir = vec![
            ColumnRef {
                name: "text_doclen".to_owned(),
                role: ROLE_TEXT_DOCLEN,
                byte_offset: doclen_off,
                byte_len: doclen_len,
                elem_count: n_docs as u64,
                width: 4,
                codec: CODEC_FIXED,
                skip_index: Vec::new(),
            },
            present_column_ref(present_off, present_len, words),
            ColumnRef {
                name: "dict".to_owned(),
                role: ROLE_DICT,
                byte_offset: dict_off,
                byte_len: dict_len,
                elem_count: dict_count,
                width: 0,
                codec: CODEC_RAW_VAR,
                skip_index: Vec::new(),
            },
            ColumnRef {
                name: "dict_offsets".to_owned(),
                role: ROLE_DICT_OFFSETS,
                byte_offset: dict_offsets_off,
                byte_len: dict_offsets_len,
                elem_count: dict_count + 1,
                width: 8,
                codec: CODEC_FIXED,
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
                skip_index: postings_skip,
            },
        ];
        let mut dir_bytes = Vec::new();
        ciborium::into_writer(&dir, &mut dir_bytes)
            .map_err(|error| anyhow!("cbor encode streamed segment directory: {error}"))?;
        let dir_offset = at;
        write_counted(&mut out, &mut at, &dir_bytes)?;
        let footer = Footer {
            dir_offset,
            dir_len: dir_bytes.len() as u64,
            crc32: crc32fast::hash(&dir_bytes),
            magic2: MAGIC2,
        };
        write_counted(&mut out, &mut at, &footer.to_bytes())?;
        out.flush()
            .with_context(|| format!("flush {}", temp.display()))?;
        out.get_ref()
            .sync_all()
            .with_context(|| format!("fsync {}", temp.display()))?;
        drop(out);
        std::fs::rename(&temp, path)
            .with_context(|| format!("rename {} -> {}", temp.display(), path.display()))?;
        Ok(())
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&temp);
    }
    result
}

/// Existing composed-reader entrypoint.
pub(crate) fn write_text_stream(path: &Path, seq: u64, view: &ComposedSegmentReader) -> Result<()> {
    write_text_projection(path, seq, view)
}

fn stream_atomic(
    path: &Path,
    write: impl FnOnce(&mut BufWriter<File>, &mut u64) -> Result<()>,
) -> Result<()> {
    let (temp, file) = new_stream_temp(path)?;
    let result = (|| {
        let mut out = BufWriter::new(file);
        let mut at = 0u64;
        write(&mut out, &mut at)?;
        out.flush()
            .with_context(|| format!("flush {}", temp.display()))?;
        out.get_ref()
            .sync_all()
            .with_context(|| format!("fsync {}", temp.display()))?;
        drop(out);
        std::fs::rename(&temp, path)
            .with_context(|| format!("rename {} -> {}", temp.display(), path.display()))?;
        Ok(())
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&temp);
    }
    result
}

fn write_stream_directory(
    out: &mut BufWriter<File>,
    at: &mut u64,
    dir: Vec<ColumnRef>,
) -> Result<()> {
    let mut bytes = Vec::new();
    ciborium::into_writer(&dir, &mut bytes)
        .map_err(|error| anyhow!("cbor encode streamed segment directory: {error}"))?;
    let footer = Footer {
        dir_offset: *at,
        dir_len: bytes.len() as u64,
        crc32: crc32fast::hash(&bytes),
        magic2: MAGIC2,
    };
    write_counted(out, at, &bytes)?;
    write_counted(out, at, &footer.to_bytes())
}

fn write_present_stream(
    out: &mut BufWriter<File>,
    at: &mut u64,
    n_docs: u32,
    mut present: impl FnMut(u32) -> bool,
) -> Result<(u64, u64, u64)> {
    let pad = (8 - (*at % 8)) % 8;
    write_zeroes(out, at, pad as usize)?;
    let offset = *at;
    let mut words = 0u64;
    let mut word = 0u64;
    for id in 0..n_docs {
        if present(id) {
            word |= 1u64 << (id % 64);
        }
        if id % 64 == 63 {
            write_counted(out, at, &word.to_le_bytes())?;
            words += 1;
            word = 0;
        }
    }
    if n_docs % 64 != 0 {
        write_counted(out, at, &word.to_le_bytes())?;
        words += 1;
    }
    Ok((offset, *at - offset, words))
}

/// A disposable, complete live dictionary encoded with the ordinary segment
/// VAR codec.  It replaces the old `rows * terms` ordinal walk with the
/// reader's binary search over bounded decoded blocks.  The file is created
/// with `create_new` and owns its own cleanup, so no other compaction temp can
/// be replaced or removed.
struct DictionarySpool {
    reader: SegmentReader,
    path: PathBuf,
    count: u64,
}

impl DictionarySpool {
    fn build(
        target: &Path,
        seq: u64,
        view: &ComposedSegmentReader,
        mut live: impl FnMut(&str) -> bool,
    ) -> Result<Self> {
        let (path, file) = new_stream_temp(target)?;
        let cleanup_path = path.clone();
        let result = (|| {
            let mut out = BufWriter::new(file);
            let mut at = 0u64;
            write_counted(&mut out, &mut at, &header_block(seq, 0, 0, 0))?;
            pad_to_page(&mut out, &mut at)?;
            let dict_start = at;
            let mut dict = StreamingVarWriter::new();
            let mut terms = view.string_terms(false)?;
            while let Some(term) = terms.next()? {
                if live(&term) {
                    dict.push(term.as_bytes(), &mut out, &mut at)?;
                }
            }
            let (skip, off, len, count) = dict.finish(&mut out, &mut at, dict_start)?;
            write_stream_directory(
                &mut out,
                &mut at,
                vec![ColumnRef {
                    name: "dict".to_owned(),
                    role: ROLE_DICT,
                    byte_offset: off,
                    byte_len: len,
                    elem_count: count,
                    width: 0,
                    codec: CODEC_LZ4_VAR,
                    skip_index: skip,
                }],
            )?;
            out.flush()
                .with_context(|| format!("flush {}", path.display()))?;
            out.get_ref()
                .sync_all()
                .with_context(|| format!("fsync {}", path.display()))?;
            drop(out);
            let reader = SegmentReader::open(&path)
                .with_context(|| format!("open dictionary spool {}", path.display()))?;
            Ok(Self {
                reader,
                path,
                count,
            })
        })();
        if result.is_err() {
            let _ = std::fs::remove_file(&cleanup_path);
        }
        result
    }

    fn dict_id(&self, value: &str) -> Result<u32> {
        #[cfg(test)]
        STREAM_SPOOL_LOOKUPS.with(|lookups| lookups.set(lookups.get() + 1));
        self.reader.keyword_dict_id(value).ok_or_else(|| {
            anyhow!(
                "composed string value missing from dictionary spool ({} bytes)",
                value.len()
            )
        })
    }
}

impl Drop for DictionarySpool {
    fn drop(&mut self) {
        // `path` came from our `create_new` call.  A failed cleanup is harmless:
        // it leaves only this process's uniquely-named temporary segment.
        let _ = std::fs::remove_file(&self.path);
    }
}

#[cfg(test)]
std::thread_local! {
    static STREAM_SPOOL_LOOKUPS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

fn encode_bitmap_posting(posting: &roaring::RoaringBitmap) -> Vec<u8> {
    let mut out = Vec::new();
    write_varint(&mut out, posting.len());
    let mut previous = 0u32;
    for id in posting {
        write_varint(&mut out, id.wrapping_sub(previous) as u64);
        previous = id;
    }
    out
}

fn inverse_sortable_bits(bits: u64) -> f64 {
    let raw = if bits >> 63 == 1 {
        bits ^ (1u64 << 63)
    } else {
        !bits
    };
    f64::from_bits(raw)
}

/// Apply-time temporary storage charged by the staging owner.  This is an
/// internal retry charge, never a public request limit.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ScalarProjectionScratch {
    max_entry_bytes: usize,
    raw_dictionary: bool,
}
impl ScalarProjectionScratch {
    pub(crate) const fn new(max_entry_bytes: usize) -> Self {
        Self {
            max_entry_bytes,
            raw_dictionary: false,
        }
    }
    /// Select the mmap-readable raw dictionary codec for a scalar projection.
    /// It is private to the projection path; existing callers retain the
    /// bounded prefix/LZ4 codec by default.
    pub(crate) const fn raw_scalar_dictionary(mut self) -> Self {
        self.raw_dictionary = true;
        self
    }
    fn require(&self, kind: &'static str, required: usize) -> Result<()> {
        if required <= self.max_entry_bytes {
            Ok(())
        } else {
            Err(ScalarProjectionScratchRequired { kind, required }.into())
        }
    }
}

/// Stream borrowed UTF-8 terms to `out` and keep only u64 entry boundaries in
/// a private spool.  The spool is copied into the final fixed column after the
/// raw bytes, so neither the writer nor the directory owns a term-sized buffer.
fn write_raw_dictionary(
    target: &Path,
    out: &mut BufWriter<File>,
    at: &mut u64,
    mut terms: impl FnMut(&mut dyn FnMut(&str) -> Result<()>) -> Result<()>,
    expected: Option<&DictionarySpool>,
) -> Result<(u64, u64, u64, u64, u64)> {
    let (offset_path, offset_file) = new_stream_temp(target)?;
    let result = (|| {
        let mut offsets = BufWriter::new(offset_file);
        let start = *at;
        offsets.write_all(&0u64.to_le_bytes())?;
        let mut count = 0u64;
        terms(&mut |term| {
            if let Some(spool) = expected {
                let ordinal = u32::try_from(count).context("raw dictionary ordinal exceeds u32")?;
                let prior = spool
                    .reader
                    .keyword_term_at_ordinal_cow(ordinal)
                    .ok_or_else(|| anyhow!("raw dictionary changed before ordinal {ordinal}"))?;
                if prior.as_bytes() != term.as_bytes() {
                    bail!("raw dictionary changed between validated spool and target write")
                }
            }
            write_counted(out, at, term.as_bytes())?;
            count = count
                .checked_add(1)
                .ok_or_else(|| anyhow!("raw dictionary exceeds u64 ordinal capacity"))?;
            if count > u64::from(u32::MAX) {
                bail!("raw dictionary exceeds u32 ordinal capacity")
            }
            offsets.write_all(&(*at - start).to_le_bytes())?;
            Ok(())
        })?;
        offsets.flush()?;
        drop(offsets);
        let data_len = *at - start;
        if expected.is_some_and(|spool| spool.count != count) {
            bail!("raw dictionary count changed between validated spool and target write")
        }
        pad_to_page(out, at)?;
        let offsets_off = *at;
        let mut source = File::open(&offset_path)
            .with_context(|| format!("open raw dictionary offsets {}", offset_path.display()))?;
        let mut buffer = [0u8; 64 * 1024];
        loop {
            let n = source.read(&mut buffer)?;
            if n == 0 {
                break;
            }
            write_counted(out, at, &buffer[..n])?;
        }
        let offsets_len = *at - offsets_off;
        if offsets_len
            != count
                .checked_add(1)
                .and_then(|n| n.checked_mul(8))
                .ok_or_else(|| anyhow!("raw dictionary offsets overflow"))?
        {
            bail!("raw dictionary offsets spool length mismatch")
        }
        Ok((start, data_len, count, offsets_off, offsets_len))
    })();
    let _ = std::fs::remove_file(&offset_path);
    result
}
#[derive(Debug)]
pub(crate) struct ScalarProjectionScratchRequired {
    pub(crate) kind: &'static str,
    pub(crate) required: usize,
}
impl std::fmt::Display for ScalarProjectionScratchRequired {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "scalar projection {} needs {} bytes of scratch",
            self.kind, self.required
        )
    }
}
impl std::error::Error for ScalarProjectionScratchRequired {}

/// Peak ownership for a projection whose VAR columns use full-entry block
/// boundaries. Counts include duplicate source terms, which safely overprice
/// dictionaries after deduplication. No term content is needed for this pass.
pub(crate) fn scalar_projection_peak_bound(
    max_entry: usize,
    term_bytes: usize,
    terms: usize,
    memberships: usize,
) -> Result<usize> {
    let overflow = || anyhow!("scalar projection memory bound overflow");
    let add = |a: usize, b: usize| a.checked_add(b).ok_or_else(overflow);
    let mul = |a: usize, b: usize| a.checked_mul(b).ok_or_else(overflow);
    // Every flushed block except the last has at least 64 KiB of full entry
    // bytes + four bytes per entry. One large entry may exceed that target.
    let block = add(VAR_BLOCK_BYTES, add(max_entry, 4)?)?;
    let entries = terms.min(VAR_BLOCK_BYTES / 4 + 1);
    // CBOR: <= 2 bytes/u8, <= 64 bytes per fixed VarEntry map, plus outer
    // framing. Vec growth is charged at twice the encoded bound. LZ4's output
    // maximum is below 2*raw+64, including the prepended length.
    let raw = add(add(mul(2, block)?, mul(64, entries)?)?, 256)?;
    let entry_vectors = mul(
        mul(2, entries)?,
        std::mem::size_of::<VarEntry>() + std::mem::size_of::<Vec<u8>>(),
    )?;
    let codec_peak = add(add(mul(8, block)?, mul(6, raw)?)?, entry_vectors)?;
    let dictionary_total = add(term_bytes, mul(4, terms)?)?;
    // Each count and each u32 delta needs at most five LEB128 bytes.
    let posting_total = add(mul(9, terms)?, mul(5, memberships)?)?;
    let dict_blocks = add(dictionary_total / VAR_BLOCK_BYTES, 1)?;
    let post_blocks = add(posting_total / VAR_BLOCK_BYTES, 1)?;
    let blocks = add(dict_blocks, post_blocks)?;
    // Each skip entry has four bounded integer fields. 128 bytes/entry plus
    // 4096 for all fixed column names, maps, arrays and final directory framing
    // overprices the existing CBOR format. The spool has a subset of this.
    let directory = add(4096, mul(128, blocks)?)?;
    let reader = add(
        std::mem::size_of::<SegmentReader>(),
        mul(
            directory,
            std::mem::size_of::<ColumnRef>() + std::mem::size_of::<VarBlockMeta>() + 8,
        )?,
    )?;
    let writer_metadata = add(
        mul(mul(2, blocks)?, std::mem::size_of::<VarBlockMeta>())?,
        mul(8, directory)?,
    )?;
    // A spool cache can retain 16 MiB while a miss decodes a new block and a
    // target writer retains its current block. Include both codec workspaces.
    add(
        add(
            add(DEFAULT_VAR_CACHE_BYTES as usize, mul(2, codec_peak)?)?,
            add(reader, writer_metadata)?,
        )?,
        128 * 1024,
    )
}

/// A scalar source supplies fresh, rewindable callback passes.  The fast WAL
/// source can therefore lend UTF-8 values directly from retained command bytes.
pub(crate) trait KeywordStreamProjection {
    fn n_docs(&self) -> u32;
    fn keyword_row(&self, row: u32, emit: &mut dyn FnMut(Option<&str>) -> Result<()>)
        -> Result<()>;
    fn keyword_terms(&self, emit: &mut dyn FnMut(&str) -> Result<()>) -> Result<()>;
    fn keyword_posting(&self, term: &str, emit: &mut dyn FnMut(u32) -> Result<()>) -> Result<bool>;
}
pub(crate) trait SetStreamProjection {
    fn n_docs(&self) -> u32;
    /// Returns false for an absent row; a present empty row calls no emit.
    fn set_row(&self, row: u32, emit: &mut dyn FnMut(&str) -> Result<()>) -> Result<bool>;
    fn set_terms(&self, emit: &mut dyn FnMut(&str) -> Result<()>) -> Result<()>;
    fn set_posting(&self, term: &str, emit: &mut dyn FnMut(u32) -> Result<()>) -> Result<bool>;
}
pub(crate) trait NumberStreamProjection {
    fn n_docs(&self) -> u32;
    fn number_row(&self, row: u32, emit: &mut dyn FnMut(Option<f64>) -> Result<()>) -> Result<()>;
    fn number_keys(&self, emit: &mut dyn FnMut(u64) -> Result<()>) -> Result<()>;
    fn number_posting(&self, key: u64, emit: &mut dyn FnMut(u32) -> Result<()>) -> Result<bool>;
}

fn leb128_len(mut value: u64) -> usize {
    let mut bytes = 1;
    while value >= 0x80 {
        value >>= 7;
        bytes += 1;
    }
    bytes
}

/// Validate a posting once, then replay it into exactly one charged legacy
/// blob.  The second pass checks capacity before every varint push.
fn projection_posting(
    n_docs: u32,
    scratch: ScalarProjectionScratch,
    kind: &'static str,
    mut visit: impl FnMut(&mut dyn FnMut(u32) -> Result<()>) -> Result<bool>,
) -> Result<Option<Vec<u8>>> {
    let mut count = 0u64;
    use sha2::{Digest, Sha256};
    let mut payload = 0usize;
    let mut expected_rows = Sha256::new();
    let mut previous = None;
    let live = visit(&mut |row| {
        if row >= n_docs || previous.is_some_and(|old| old >= row) {
            bail!("scalar projection {kind} has unsorted or out-of-range posting row");
        }
        expected_rows.update(row.to_le_bytes());
        payload = payload
            .checked_add(leb128_len(u64::from(row - previous.unwrap_or(0))))
            .ok_or_else(|| anyhow!("scalar projection posting length overflow"))?;
        previous = Some(row);
        count = count
            .checked_add(1)
            .ok_or_else(|| anyhow!("scalar projection posting count overflow"))?;
        Ok(())
    })?;
    if !live {
        if count != 0 {
            bail!("scalar projection {kind} reported absent posting with rows")
        }
        return Ok(None);
    }
    if count == 0 {
        bail!("scalar projection {kind} reported live empty posting");
    }
    let required = leb128_len(count)
        .checked_add(payload)
        .ok_or_else(|| anyhow!("scalar projection posting length overflow"))?;
    scratch.require(kind, required)?;
    let mut blob = Vec::with_capacity(required);
    if leb128_len(count) > required {
        bail!("scalar projection {kind} changed between rewindable posting passes");
    }
    write_varint(&mut blob, count);
    let mut previous = None;
    let mut seen = 0u64;
    let mut actual_rows = Sha256::new();
    let replayed = visit(&mut |row| {
        if row >= n_docs || previous.is_some_and(|old| old >= row) {
            bail!("scalar projection {kind} changed or unsorted its replay posting");
        }
        let width = leb128_len(u64::from(row - previous.unwrap_or(0)));
        if blob
            .len()
            .checked_add(width)
            .map_or(true, |next| next > required)
        {
            bail!("scalar projection {kind} changed between rewindable posting passes");
        }
        actual_rows.update(row.to_le_bytes());
        write_varint(&mut blob, u64::from(row - previous.unwrap_or(0)));
        previous = Some(row);
        seen += 1;
        Ok(())
    })?;
    if !replayed
        || seen != count
        || blob.len() != required
        || expected_rows.finalize() != actual_rows.finalize()
    {
        bail!("scalar projection {kind} changed between rewindable posting passes");
    }
    Ok(Some(blob))
}

fn write_present_projection(
    out: &mut BufWriter<File>,
    at: &mut u64,
    n_docs: u32,
    mut present: impl FnMut(u32) -> Result<bool>,
) -> Result<(u64, u64, u64)> {
    let pad = (8 - (*at % 8)) % 8;
    write_zeroes(out, at, pad as usize)?;
    let offset = *at;
    let mut words = 0u64;
    let mut word = 0u64;
    for id in 0..n_docs {
        if present(id)? {
            word |= 1u64 << (id % 64);
        }
        if id % 64 == 63 {
            write_counted(out, at, &word.to_le_bytes())?;
            words += 1;
            word = 0;
        }
    }
    if n_docs % 64 != 0 {
        write_counted(out, at, &word.to_le_bytes())?;
        words += 1;
    }
    Ok((offset, *at - offset, words))
}

fn projection_dictionary(
    target: &Path,
    seq: u64,
    scratch: ScalarProjectionScratch,
    mut terms: impl FnMut(&mut dyn FnMut(&str) -> Result<()>) -> Result<()>,
) -> Result<DictionarySpool> {
    if scratch.raw_dictionary {
        return projection_raw_dictionary(target, seq, terms);
    }
    let (path, file) = new_stream_temp(target)?;
    let cleanup = path.clone();
    let result = (|| {
        let mut out = BufWriter::new(file);
        let mut at = 0u64;
        write_counted(&mut out, &mut at, &header_block(seq, 0, 0, 0))?;
        pad_to_page(&mut out, &mut at)?;
        let start = at;
        let mut dict = StreamingVarWriter::bounded_projection();
        let mut last = Vec::new();
        let mut has_last = false;
        terms(&mut |term| {
            scratch.require("dictionary term", term.len())?;
            if has_last && last.as_slice() >= term.as_bytes() {
                bail!("scalar projection terms are not strictly sorted");
            }
            last.clear();
            last.extend_from_slice(term.as_bytes());
            has_last = true;
            dict.push(term.as_bytes(), &mut out, &mut at)
        })?;
        let (skip, off, len, count) = dict.finish(&mut out, &mut at, start)?;
        write_stream_directory(
            &mut out,
            &mut at,
            vec![ColumnRef {
                name: "dict".to_owned(),
                role: ROLE_DICT,
                byte_offset: off,
                byte_len: len,
                elem_count: count,
                width: 0,
                codec: CODEC_LZ4_VAR,
                skip_index: skip,
            }],
        )?;
        out.flush()?;
        out.get_ref().sync_all()?;
        drop(out);
        Ok(DictionarySpool {
            reader: SegmentReader::open(&path)?,
            path,
            count,
        })
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(cleanup);
    }
    result
}

fn projection_raw_dictionary(
    target: &Path,
    seq: u64,
    terms: impl FnMut(&mut dyn FnMut(&str) -> Result<()>) -> Result<()>,
) -> Result<DictionarySpool> {
    let (path, file) = new_stream_temp(target)?;
    let cleanup = path.clone();
    let result = (|| {
        let mut out = BufWriter::new(file);
        let mut at = 0u64;
        write_counted(&mut out, &mut at, &header_block(seq, 0, 0, 0))?;
        pad_to_page(&mut out, &mut at)?;
        let (dict_off, dict_len, count, offsets_off, offsets_len) =
            write_raw_dictionary(target, &mut out, &mut at, terms, None)?;
        write_stream_directory(
            &mut out,
            &mut at,
            vec![
                ColumnRef {
                    name: "dict".to_owned(),
                    role: ROLE_DICT,
                    byte_offset: dict_off,
                    byte_len: dict_len,
                    elem_count: count,
                    width: 0,
                    codec: CODEC_RAW_VAR,
                    skip_index: Vec::new(),
                },
                ColumnRef {
                    name: "dict_offsets".to_owned(),
                    role: ROLE_DICT_OFFSETS,
                    byte_offset: offsets_off,
                    byte_len: offsets_len,
                    elem_count: count + 1,
                    width: 8,
                    codec: CODEC_FIXED,
                    skip_index: Vec::new(),
                },
            ],
        )?;
        out.flush()?;
        out.get_ref().sync_all()?;
        drop(out);
        Ok(DictionarySpool {
            reader: SegmentReader::open(&path)?,
            path,
            count,
        })
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(cleanup);
    }
    result
}

fn write_projection_dictionary(
    target: &Path,
    scratch: ScalarProjectionScratch,
    out: &mut BufWriter<File>,
    at: &mut u64,
    mut terms: impl FnMut(&mut dyn FnMut(&str) -> Result<()>) -> Result<()>,
    expected: Option<&DictionarySpool>,
) -> Result<(Vec<ColumnRef>, u64)> {
    if scratch.raw_dictionary {
        let (off, len, count, offsets_off, offsets_len) =
            write_raw_dictionary(target, out, at, terms, expected)?;
        return Ok((
            vec![
                ColumnRef {
                    name: "dict".to_owned(),
                    role: ROLE_DICT,
                    byte_offset: off,
                    byte_len: len,
                    elem_count: count,
                    width: 0,
                    codec: CODEC_RAW_VAR,
                    skip_index: Vec::new(),
                },
                ColumnRef {
                    name: "dict_offsets".to_owned(),
                    role: ROLE_DICT_OFFSETS,
                    byte_offset: offsets_off,
                    byte_len: offsets_len,
                    elem_count: count + 1,
                    width: 8,
                    codec: CODEC_FIXED,
                    skip_index: Vec::new(),
                },
            ],
            count,
        ));
    }
    let start = *at;
    let mut writer = StreamingVarWriter::bounded_projection();
    terms(&mut |term| {
        scratch.require("dictionary term", term.len())?;
        writer.push(term.as_bytes(), out, at)
    })?;
    let (skip, off, len, count) = writer.finish(out, at, start)?;
    Ok((
        vec![ColumnRef {
            name: "dict".to_owned(),
            role: ROLE_DICT,
            byte_offset: off,
            byte_len: len,
            elem_count: count,
            width: 0,
            codec: CODEC_LZ4_VAR,
            skip_index: skip,
        }],
        count,
    ))
}

pub(crate) fn write_keyword_projection<V: KeywordStreamProjection>(
    path: &Path,
    seq: u64,
    view: &V,
    scratch: ScalarProjectionScratch,
) -> Result<()> {
    let n_docs = view.n_docs();
    let spool = projection_dictionary(path, seq, scratch, |emit| view.keyword_terms(emit))?;
    stream_atomic(path, |out, at| {
        write_counted(out, at, &header_block(seq, n_docs, 0, 0))?;
        let dictid_off = *at;
        for id in 0..n_docs {
            let mut ordinal = None;
            view.keyword_row(id, &mut |value| {
                if ordinal.is_some() {
                    bail!("keyword projection emitted a row more than once");
                }
                ordinal = Some(match value {
                    Some(value) => {
                        if !scratch.raw_dictionary {
                            scratch.require("keyword row", value.len())?;
                        }
                        spool.dict_id(value)?
                    }
                    None => DICT_ABSENT,
                });
                Ok(())
            })?;
            let ordinal = ordinal.ok_or_else(|| anyhow!("keyword projection omitted a row"))?;
            write_counted(out, at, &ordinal.to_le_bytes())?;
        }
        let dictid_len = *at - dictid_off;
        let (present_off, present_len, words) = write_present_projection(out, at, n_docs, |id| {
            let mut yes = false;
            view.keyword_row(id, &mut |v| {
                yes = v.is_some();
                Ok(())
            })?;
            Ok(yes)
        })?;
        pad_to_page(out, at)?;
        let (dict_columns, dict_count) = write_projection_dictionary(
            path,
            scratch,
            out,
            at,
            |emit| view.keyword_terms(emit),
            Some(&spool),
        )?;
        let posting_start = *at;
        let mut postings = StreamingVarWriter::bounded_projection();
        view.keyword_terms(&mut |term| {
            if let Some(blob) = projection_posting(n_docs, scratch, "keyword posting", |emit| {
                view.keyword_posting(term, emit)
            })? {
                postings.push(&blob, out, at)?;
            }
            Ok(())
        })?;
        let (posting_skip, posting_off, posting_len, posting_count) =
            postings.finish(out, at, posting_start)?;
        if dict_count != posting_count || dict_count != spool.count {
            bail!("streamed keyword dictionary/posting ordinal mismatch");
        }
        let mut directory = vec![
            ColumnRef {
                name: "keyword_dictid".to_owned(),
                role: ROLE_KEYWORD_DICTID,
                byte_offset: dictid_off,
                byte_len: dictid_len,
                elem_count: n_docs as u64,
                width: 4,
                codec: CODEC_FIXED,
                skip_index: Vec::new(),
            },
            present_column_ref(present_off, present_len, words),
        ];
        directory.extend(dict_columns);
        directory.push(ColumnRef {
            name: "keyword_postings".to_owned(),
            role: ROLE_KEYWORD_POSTINGS,
            byte_offset: posting_off,
            byte_len: posting_len,
            elem_count: posting_count,
            width: 0,
            codec: CODEC_LZ4_VAR,
            skip_index: posting_skip,
        });
        write_stream_directory(out, at, directory)
    })
}

pub(crate) fn write_set_projection<V: SetStreamProjection>(
    path: &Path,
    seq: u64,
    view: &V,
    scratch: ScalarProjectionScratch,
) -> Result<()> {
    let n_docs = view.n_docs();
    let spool = projection_dictionary(path, seq, scratch, |emit| view.set_terms(emit))?;
    stream_atomic(path, |out, at| {
        write_counted(out, at, &header_block(seq, n_docs, 0, 0))?;
        let offsets_off = *at;
        let mut packed = 0u32;
        write_counted(out, at, &packed.to_le_bytes())?;
        for id in 0..n_docs {
            let mut members = 0u32;
            let mut previous = None;
            let present = view.set_row(id, &mut |member| {
                let ordinal = spool.dict_id(member)?;
                if previous.is_some_and(|old| old >= ordinal) {
                    bail!("set projection members must be sorted and unique");
                }
                previous = Some(ordinal);
                members = members
                    .checked_add(1)
                    .ok_or_else(|| anyhow!("set member count exceeds u32"))?;
                Ok(())
            })?;
            if !present && members != 0 {
                bail!("set projection emitted members for an absent row");
            }
            if present {
                packed = packed
                    .checked_add(members)
                    .ok_or_else(|| anyhow!("set packed column exceeds u32"))?;
            }
            write_counted(out, at, &packed.to_le_bytes())?;
        }
        let offsets_len = *at - offsets_off;
        let packed_off = *at;
        for id in 0..n_docs {
            view.set_row(id, &mut |member| {
                write_counted(out, at, &spool.dict_id(member)?.to_le_bytes())
            })?;
        }
        let packed_len = *at - packed_off;
        if packed_len != u64::from(packed) * 4 {
            bail!("set projection changed its packed row count during replay");
        }
        let (present_off, present_len, words) =
            write_present_projection(out, at, n_docs, |id| view.set_row(id, &mut |_| Ok(())))?;
        pad_to_page(out, at)?;
        let (dict_columns, dict_count) = write_projection_dictionary(
            path,
            scratch,
            out,
            at,
            |emit| view.set_terms(emit),
            Some(&spool),
        )?;
        let posting_start = *at;
        let mut postings = StreamingVarWriter::bounded_projection();
        view.set_terms(&mut |term| {
            if let Some(blob) = projection_posting(n_docs, scratch, "set posting", |emit| {
                view.set_posting(term, emit)
            })? {
                postings.push(&blob, out, at)?;
            }
            Ok(())
        })?;
        let (posting_skip, posting_off, posting_len, posting_count) =
            postings.finish(out, at, posting_start)?;
        if dict_count != posting_count || dict_count != spool.count {
            bail!("streamed set dictionary/posting ordinal mismatch");
        }
        let mut directory = vec![
            ColumnRef {
                name: "set_offsets".to_owned(),
                role: ROLE_SET_OFFSETS,
                byte_offset: offsets_off,
                byte_len: offsets_len,
                elem_count: n_docs as u64 + 1,
                width: 4,
                codec: CODEC_FIXED,
                skip_index: Vec::new(),
            },
            ColumnRef {
                name: "set_packed".to_owned(),
                role: ROLE_SET_PACKED,
                byte_offset: packed_off,
                byte_len: packed_len,
                elem_count: packed as u64,
                width: 4,
                codec: CODEC_FIXED,
                skip_index: Vec::new(),
            },
            present_column_ref(present_off, present_len, words),
        ];
        directory.extend(dict_columns);
        directory.push(ColumnRef {
            name: "set_postings".to_owned(),
            role: ROLE_SET_POSTINGS,
            byte_offset: posting_off,
            byte_len: posting_len,
            elem_count: posting_count,
            width: 0,
            codec: CODEC_LZ4_VAR,
            skip_index: posting_skip,
        });
        write_stream_directory(out, at, directory)
    })
}

pub(crate) fn write_number_projection<V: NumberStreamProjection>(
    path: &Path,
    seq: u64,
    view: &V,
    scratch: ScalarProjectionScratch,
) -> Result<()> {
    let n_docs = view.n_docs();
    stream_atomic(path, |out, at| {
        write_counted(out, at, &header_block(seq, n_docs, 0, 0))?;
        let number_off = *at;
        for id in 0..n_docs {
            let mut value = None;
            view.number_row(id, &mut |v| {
                value = v;
                Ok(())
            })?;
            write_counted(out, at, &value.map(f64::to_bits).unwrap_or(0).to_le_bytes())?;
        }
        let number_len = *at - number_off;
        let sorted_off = *at;
        let mut count = 0u64;
        let mut last = None;
        view.number_keys(&mut |key| {
            if last.is_some_and(|old| old >= key)
                || sortable_bits(inverse_sortable_bits(key)) != key
            {
                bail!("invalid scalar projection sortable number key");
            }
            let live = view.number_posting(key, &mut |_| Ok(()))?;
            if live {
                write_counted(out, at, &key.to_le_bytes())?;
                count += 1;
            }
            last = Some(key);
            Ok(())
        })?;
        let sorted_len = *at - sorted_off;
        let (present_off, present_len, words) = write_present_projection(out, at, n_docs, |id| {
            let mut yes = false;
            view.number_row(id, &mut |v| {
                yes = v.is_some();
                Ok(())
            })?;
            Ok(yes)
        })?;
        pad_to_page(out, at)?;
        let posting_start = *at;
        let mut postings = StreamingVarWriter::bounded_projection();
        view.number_keys(&mut |key| {
            if let Some(blob) = projection_posting(n_docs, scratch, "number posting", |emit| {
                view.number_posting(key, emit)
            })? {
                postings.push(&blob, out, at)?;
            }
            Ok(())
        })?;
        let (posting_skip, posting_off, posting_len, posting_count) =
            postings.finish(out, at, posting_start)?;
        if count != posting_count {
            bail!("streamed number key/posting ordinal mismatch");
        }
        write_stream_directory(
            out,
            at,
            vec![
                ColumnRef {
                    name: "number".to_owned(),
                    role: ROLE_NUMBER,
                    byte_offset: number_off,
                    byte_len: number_len,
                    elem_count: n_docs as u64,
                    width: 8,
                    codec: CODEC_FIXED,
                    skip_index: Vec::new(),
                },
                ColumnRef {
                    name: "number_sorted".to_owned(),
                    role: ROLE_NUMBER_SORTED,
                    byte_offset: sorted_off,
                    byte_len: sorted_len,
                    elem_count: count,
                    width: 8,
                    codec: CODEC_FIXED,
                    skip_index: Vec::new(),
                },
                present_column_ref(present_off, present_len, words),
                ColumnRef {
                    name: "number_postings".to_owned(),
                    role: ROLE_NUMBER_POSTINGS,
                    byte_offset: posting_off,
                    byte_len: posting_len,
                    elem_count: posting_count,
                    width: 0,
                    codec: CODEC_LZ4_VAR,
                    skip_index: posting_skip,
                },
            ],
        )
    })
}

impl KeywordStreamProjection for ComposedSegmentReader {
    fn n_docs(&self) -> u32 {
        ComposedSegmentReader::n_docs(self)
    }
    fn keyword_row(
        &self,
        row: u32,
        emit: &mut dyn FnMut(Option<&str>) -> Result<()>,
    ) -> Result<()> {
        let value = self.keyword_at_cow(row);
        emit(value.as_deref())
    }
    fn keyword_terms(&self, emit: &mut dyn FnMut(&str) -> Result<()>) -> Result<()> {
        let mut terms = self.string_terms(false)?;
        while let Some(term) = terms.next_cow()? {
            if self.keyword_postings(term.as_ref()).is_some() {
                emit(term.as_ref())?;
            }
        }
        Ok(())
    }
    fn keyword_posting(&self, term: &str, emit: &mut dyn FnMut(u32) -> Result<()>) -> Result<bool> {
        match self.keyword_postings(term) {
            Some(p) => {
                for row in p {
                    emit(row)?;
                }
                Ok(true)
            }
            None => Ok(false),
        }
    }
}
impl SetStreamProjection for ComposedSegmentReader {
    fn n_docs(&self) -> u32 {
        ComposedSegmentReader::n_docs(self)
    }
    fn set_row(&self, row: u32, emit: &mut dyn FnMut(&str) -> Result<()>) -> Result<bool> {
        let Some((present, count)) = self.set_row_member_count(row) else {
            return Ok(false);
        };
        if !present {
            return Ok(false);
        }
        for member in 0..count {
            let value = self
                .set_member_at_cow(row, member)
                .ok_or_else(|| anyhow!("invalid set member in scalar projection"))?;
            emit(value.as_ref())?;
        }
        Ok(true)
    }
    fn set_terms(&self, emit: &mut dyn FnMut(&str) -> Result<()>) -> Result<()> {
        let mut terms = self.string_terms(false)?;
        while let Some(term) = terms.next_cow()? {
            if self.set_postings(term.as_ref()).is_some() {
                emit(term.as_ref())?;
            }
        }
        Ok(())
    }
    fn set_posting(&self, term: &str, emit: &mut dyn FnMut(u32) -> Result<()>) -> Result<bool> {
        match self.set_postings(term) {
            Some(p) => {
                for row in p {
                    emit(row)?;
                }
                Ok(true)
            }
            None => Ok(false),
        }
    }
}
impl NumberStreamProjection for ComposedSegmentReader {
    fn n_docs(&self) -> u32 {
        ComposedSegmentReader::n_docs(self)
    }
    fn number_row(&self, row: u32, emit: &mut dyn FnMut(Option<f64>) -> Result<()>) -> Result<()> {
        emit(self.number_at(row))
    }
    fn number_keys(&self, emit: &mut dyn FnMut(u64) -> Result<()>) -> Result<()> {
        let mut keys = self.number_keys(None, None, false)?;
        while let Some(key) = keys.next()? {
            if self.number_value_postings(key).is_some() {
                emit(key)?;
            }
        }
        Ok(())
    }
    fn number_posting(&self, key: u64, emit: &mut dyn FnMut(u32) -> Result<()>) -> Result<bool> {
        match self.number_value_postings(key) {
            Some(p) => {
                for row in p {
                    emit(row)?;
                }
                Ok(true)
            }
            None => Ok(false),
        }
    }
}

/// Stream the effective Keyword field from `view`.  A disposable mmap spool
/// supplies each forward dict-id through the segment reader's O(log terms)
/// binary lookup without a resident whole-dictionary map.
pub(crate) fn write_keyword_stream(
    path: &Path,
    seq: u64,
    view: &ComposedSegmentReader,
) -> Result<()> {
    let n_docs = view.n_docs();
    let spool = DictionarySpool::build(path, seq, view, |term| {
        view.keyword_postings(term).is_some()
    })?;
    stream_atomic(path, |out, at| {
        write_counted(out, at, &header_block(seq, n_docs, 0, 0))?;
        let dictid_off = *at;
        for id in 0..n_docs {
            let dict_id = match view.keyword_at(id) {
                Some(value) => spool.dict_id(&value)?,
                None => DICT_ABSENT,
            };
            write_counted(out, at, &dict_id.to_le_bytes())?;
        }
        let dictid_len = *at - dictid_off;
        let (present_off, present_len, words) =
            write_present_stream(out, at, n_docs, |id| view.keyword_at(id).is_some())?;
        pad_to_page(out, at)?;

        let dict_start = *at;
        let mut dict = StreamingVarWriter::new();
        let mut terms = view.string_terms(false)?;
        while let Some(term) = terms.next()? {
            if view.keyword_postings(&term).is_some() {
                dict.push(term.as_bytes(), out, at)?;
            }
        }
        let (dict_skip, dict_off, dict_len, dict_count) = dict.finish(out, at, dict_start)?;
        let posting_start = *at;
        let mut postings = StreamingVarWriter::new();
        let mut terms = view.string_terms(false)?;
        while let Some(term) = terms.next()? {
            let Some(posting) = view.keyword_postings(&term) else {
                continue;
            };
            let blob = encode_bitmap_posting(&posting);
            postings.push(&blob, out, at)?;
        }
        let (posting_skip, posting_off, posting_len, posting_count) =
            postings.finish(out, at, posting_start)?;
        if dict_count != posting_count || dict_count != spool.count {
            bail!("streamed keyword dictionary/posting ordinal mismatch")
        }
        write_stream_directory(
            out,
            at,
            vec![
                ColumnRef {
                    name: "keyword_dictid".to_owned(),
                    role: ROLE_KEYWORD_DICTID,
                    byte_offset: dictid_off,
                    byte_len: dictid_len,
                    elem_count: n_docs as u64,
                    width: 4,
                    codec: CODEC_FIXED,
                    skip_index: Vec::new(),
                },
                present_column_ref(present_off, present_len, words),
                ColumnRef {
                    name: "dict".to_owned(),
                    role: ROLE_DICT,
                    byte_offset: dict_off,
                    byte_len: dict_len,
                    elem_count: dict_count,
                    width: 0,
                    codec: CODEC_LZ4_VAR,
                    skip_index: dict_skip,
                },
                ColumnRef {
                    name: "keyword_postings".to_owned(),
                    role: ROLE_KEYWORD_POSTINGS,
                    byte_offset: posting_off,
                    byte_len: posting_len,
                    elem_count: posting_count,
                    width: 0,
                    codec: CODEC_LZ4_VAR,
                    skip_index: posting_skip,
                },
            ],
        )
    })
}

/// Stream the effective Set field.  CSR offsets and packed IDs are emitted in
/// row order; an explicit empty set stays present while an absent row is clear.
pub(crate) fn write_set_stream(path: &Path, seq: u64, view: &ComposedSegmentReader) -> Result<()> {
    let n_docs = view.n_docs();
    let spool = DictionarySpool::build(path, seq, view, |term| view.set_postings(term).is_some())?;
    stream_atomic(path, |out, at| {
        write_counted(out, at, &header_block(seq, n_docs, 0, 0))?;
        let offsets_off = *at;
        let mut packed_count = 0u32;
        write_counted(out, at, &packed_count.to_le_bytes())?;
        for id in 0..n_docs {
            if let Some(members) = view.set_at(id) {
                packed_count = packed_count
                    .checked_add(
                        u32::try_from(members.len()).context("set member count exceeds u32")?,
                    )
                    .ok_or_else(|| anyhow!("set packed column exceeds u32"))?;
            }
            write_counted(out, at, &packed_count.to_le_bytes())?;
        }
        let offsets_len = *at - offsets_off;
        let packed_off = *at;
        for id in 0..n_docs {
            if let Some(members) = view.set_at(id) {
                for member in members {
                    let ordinal = spool.dict_id(&member)?;
                    write_counted(out, at, &ordinal.to_le_bytes())?;
                }
            }
        }
        let packed_len = *at - packed_off;
        let (present_off, present_len, words) =
            write_present_stream(out, at, n_docs, |id| view.set_at(id).is_some())?;
        pad_to_page(out, at)?;
        let dict_start = *at;
        let mut dict = StreamingVarWriter::new();
        let mut terms = view.string_terms(false)?;
        while let Some(term) = terms.next()? {
            if view.set_postings(&term).is_some() {
                dict.push(term.as_bytes(), out, at)?;
            }
        }
        let (dict_skip, dict_off, dict_len, dict_count) = dict.finish(out, at, dict_start)?;
        let posting_start = *at;
        let mut postings = StreamingVarWriter::new();
        let mut terms = view.string_terms(false)?;
        while let Some(term) = terms.next()? {
            let Some(posting) = view.set_postings(&term) else {
                continue;
            };
            let blob = encode_bitmap_posting(&posting);
            postings.push(&blob, out, at)?;
        }
        let (posting_skip, posting_off, posting_len, posting_count) =
            postings.finish(out, at, posting_start)?;
        if dict_count != posting_count || dict_count != spool.count {
            bail!("streamed set dictionary/posting ordinal mismatch")
        }
        write_stream_directory(
            out,
            at,
            vec![
                ColumnRef {
                    name: "set_offsets".to_owned(),
                    role: ROLE_SET_OFFSETS,
                    byte_offset: offsets_off,
                    byte_len: offsets_len,
                    elem_count: n_docs as u64 + 1,
                    width: 4,
                    codec: CODEC_FIXED,
                    skip_index: Vec::new(),
                },
                ColumnRef {
                    name: "set_packed".to_owned(),
                    role: ROLE_SET_PACKED,
                    byte_offset: packed_off,
                    byte_len: packed_len,
                    elem_count: packed_count as u64,
                    width: 4,
                    codec: CODEC_FIXED,
                    skip_index: Vec::new(),
                },
                present_column_ref(present_off, present_len, words),
                ColumnRef {
                    name: "dict".to_owned(),
                    role: ROLE_DICT,
                    byte_offset: dict_off,
                    byte_len: dict_len,
                    elem_count: dict_count,
                    width: 0,
                    codec: CODEC_LZ4_VAR,
                    skip_index: dict_skip,
                },
                ColumnRef {
                    name: "set_postings".to_owned(),
                    role: ROLE_SET_POSTINGS,
                    byte_offset: posting_off,
                    byte_len: posting_len,
                    elem_count: posting_count,
                    width: 0,
                    codec: CODEC_LZ4_VAR,
                    skip_index: posting_skip,
                },
            ],
        )
    })
}

/// Stream Number forward values, sorted keys, and one docid posting at a time.
pub(crate) fn write_number_stream(
    path: &Path,
    seq: u64,
    view: &ComposedSegmentReader,
) -> Result<()> {
    let n_docs = view.n_docs();
    stream_atomic(path, |out, at| {
        write_counted(out, at, &header_block(seq, n_docs, 0, 0))?;
        let number_off = *at;
        for id in 0..n_docs {
            write_counted(
                out,
                at,
                &view
                    .number_at(id)
                    .map(f64::to_bits)
                    .unwrap_or(0)
                    .to_le_bytes(),
            )?;
        }
        let number_len = *at - number_off;
        let sorted_off = *at;
        let mut keys = view.number_keys(None, None, false)?;
        let mut sorted_count = 0u64;
        while let Some(key) = keys.next()? {
            if view.number_value_postings(key).is_some() {
                // Validate the frozen sortable encoding before persisting it.
                if sortable_bits(inverse_sortable_bits(key)) != key {
                    bail!("invalid composed sortable number key")
                }
                write_counted(out, at, &key.to_le_bytes())?;
                sorted_count += 1;
            }
        }
        let sorted_len = *at - sorted_off;
        let (present_off, present_len, words) =
            write_present_stream(out, at, n_docs, |id| view.number_at(id).is_some())?;
        pad_to_page(out, at)?;
        let posting_start = *at;
        let mut postings = StreamingVarWriter::new();
        let mut keys = view.number_keys(None, None, false)?;
        while let Some(key) = keys.next()? {
            let Some(posting) = view.number_value_postings(key) else {
                continue;
            };
            let blob = encode_bitmap_posting(&posting);
            postings.push(&blob, out, at)?;
        }
        let (posting_skip, posting_off, posting_len, posting_count) =
            postings.finish(out, at, posting_start)?;
        if sorted_count != posting_count {
            bail!("streamed number key/posting ordinal mismatch")
        }
        write_stream_directory(
            out,
            at,
            vec![
                ColumnRef {
                    name: "number".to_owned(),
                    role: ROLE_NUMBER,
                    byte_offset: number_off,
                    byte_len: number_len,
                    elem_count: n_docs as u64,
                    width: 8,
                    codec: CODEC_FIXED,
                    skip_index: Vec::new(),
                },
                ColumnRef {
                    name: "number_sorted".to_owned(),
                    role: ROLE_NUMBER_SORTED,
                    byte_offset: sorted_off,
                    byte_len: sorted_len,
                    elem_count: sorted_count,
                    width: 8,
                    codec: CODEC_FIXED,
                    skip_index: Vec::new(),
                },
                present_column_ref(present_off, present_len, words),
                ColumnRef {
                    name: "number_postings".to_owned(),
                    role: ROLE_NUMBER_POSTINGS,
                    byte_offset: posting_off,
                    byte_len: posting_len,
                    elem_count: posting_count,
                    width: 0,
                    codec: CODEC_LZ4_VAR,
                    skip_index: posting_skip,
                },
            ],
        )
    })
}

/// Stream the effective Hash forward column and its presence bits.
pub(crate) fn write_hash_stream(path: &Path, seq: u64, view: &ComposedSegmentReader) -> Result<()> {
    let n_docs = view.n_docs();
    stream_atomic(path, |out, at| {
        write_counted(out, at, &header_block(seq, n_docs, 0, 0))?;
        let hash_off = *at;
        for id in 0..n_docs {
            write_counted(out, at, &view.hash_at(id).unwrap_or(0).to_le_bytes())?;
        }
        let hash_len = *at - hash_off;
        let (present_off, present_len, words) =
            write_present_stream(out, at, n_docs, |id| view.hash_at(id).is_some())?;
        write_stream_directory(
            out,
            at,
            vec![
                ColumnRef {
                    name: "hash".to_owned(),
                    role: ROLE_HASH,
                    byte_offset: hash_off,
                    byte_len: hash_len,
                    elem_count: n_docs as u64,
                    width: 8,
                    codec: CODEC_FIXED,
                    skip_index: Vec::new(),
                },
                present_column_ref(present_off, present_len, words),
            ],
        )
    })
}

/// Stream a dense Vector field without constructing `n_docs * dim` values in
/// memory. `row` is called exactly once for each dense target row and may hold
/// only that row's decoded vector. `None` writes `dim` zero components and a
/// clear present bit; `Some(vec![0.0; dim])` stays explicitly present.
pub(crate) fn write_vector_stream(
    path: &Path,
    seq: u64,
    n_docs: u32,
    dim: usize,
    mut row: impl FnMut(u32) -> Result<Option<Vec<f32>>>,
) -> Result<()> {
    if dim == 0 {
        bail!("vector segment dim must be > 0");
    }
    let dim_u64 = u64::try_from(dim).context("vector dimension exceeds u64")?;
    let elem_count = u64::from(n_docs)
        .checked_mul(dim_u64)
        .ok_or_else(|| anyhow!("vector segment element count overflow"))?;
    let vector_bytes = elem_count
        .checked_mul(std::mem::size_of::<f32>() as u64)
        .ok_or_else(|| anyhow!("vector segment byte count overflow"))?;
    let row_bytes = usize::try_from(
        dim_u64
            .checked_mul(4)
            .ok_or_else(|| anyhow!("vector row byte count overflow"))?,
    )
    .context("vector row byte count exceeds platform size")?;
    let word_count = usize::try_from((u64::from(n_docs) + 63) / 64)
        .context("vector present bitset exceeds platform size")?;

    stream_atomic(path, |out, at| {
        write_counted(out, at, &header_block(seq, n_docs, 0, 0))?;
        let vector_off = *at;
        let mut present = vec![0u64; word_count];
        for id in 0..n_docs {
            match row(id)? {
                None => write_zeroes(out, at, row_bytes)?,
                Some(values) => {
                    if values.len() != dim {
                        bail!("vector has dim {} but segment dim is {dim}", values.len());
                    }
                    for value in values {
                        if !value.is_finite() {
                            bail!("vector contains non-finite component");
                        }
                        write_counted(out, at, &value.to_le_bytes())?;
                    }
                    present[id as usize / 64] |= 1u64 << (id % 64);
                }
            }
        }
        if *at - vector_off != vector_bytes {
            bail!("streamed vector byte count mismatch");
        }
        let vector_len = *at - vector_off;
        let present_pad = (8 - (*at % 8)) % 8;
        write_zeroes(out, at, present_pad as usize)?;
        let present_off = *at;
        for word in present {
            write_counted(out, at, &word.to_le_bytes())?;
        }
        let present_len = *at - present_off;
        pad_to_page(out, at)?;
        write_stream_directory(
            out,
            at,
            vec![
                ColumnRef {
                    name: "vector".to_owned(),
                    role: ROLE_VECTOR,
                    byte_offset: vector_off,
                    byte_len: vector_len,
                    elem_count,
                    width: 4,
                    codec: CODEC_FIXED,
                    skip_index: Vec::new(),
                },
                present_column_ref(present_off, present_len, word_count as u64),
            ],
        )
    })
}

#[cfg(test)]
mod tests {
    struct BorrowedKeywordRows<'a>(&'a [Option<&'a str>]);

    impl super::KeywordStreamProjection for BorrowedKeywordRows<'_> {
        fn n_docs(&self) -> u32 {
            self.0.len() as u32
        }

        fn keyword_row(
            &self,
            row: u32,
            emit: &mut dyn FnMut(Option<&str>) -> anyhow::Result<()>,
        ) -> anyhow::Result<()> {
            emit(self.0[row as usize])
        }

        fn keyword_terms(
            &self,
            emit: &mut dyn FnMut(&str) -> anyhow::Result<()>,
        ) -> anyhow::Result<()> {
            // Test-only ordering keeps references, never copies value bytes.
            let terms: std::collections::BTreeSet<&str> =
                self.0.iter().flatten().copied().collect();
            for term in terms {
                emit(term)?;
            }
            Ok(())
        }

        fn keyword_posting(
            &self,
            term: &str,
            emit: &mut dyn FnMut(u32) -> anyhow::Result<()>,
        ) -> anyhow::Result<bool> {
            let mut found = false;
            for (row, value) in self.0.iter().enumerate() {
                if *value == Some(term) {
                    emit(row as u32)?;
                    found = true;
                }
            }
            Ok(found)
        }
    }
    use super::*;
    use crate::segment::{
        write_hash_segment, write_keyword_segment, write_number_segment, write_set_segment,
        write_text_segment, SegmentReader,
    };
    use crate::storage::Postings;
    use std::collections::BTreeMap;
    use std::sync::Arc;

    fn source(path: &Path, rows: &[Option<&[(&str, u32)]>]) -> Arc<SegmentReader> {
        let mut tokens = BTreeMap::<String, Postings>::new();
        let mut lens = Vec::new();
        for (id, row) in rows.iter().enumerate() {
            let mut len = 0;
            for &(term, tf) in row.unwrap_or_default() {
                tokens
                    .entry(term.to_owned())
                    .or_default()
                    .upsert(id as u32, tf);
                len += tf;
            }
            lens.push(len);
        }
        let present: Vec<_> = rows.iter().map(Option::is_some).collect();
        write_text_segment(
            path,
            4,
            &tokens,
            &lens,
            &present,
            present.iter().filter(|&&p| p).count() as u64,
            lens.iter().map(|&len| len as u64).sum(),
        )
        .unwrap();
        Arc::new(SegmentReader::open(path).unwrap())
    }

    fn keyword_source(path: &Path, values: &[Option<&str>]) -> Arc<SegmentReader> {
        let mut postings = BTreeMap::new();
        for (id, value) in values.iter().enumerate() {
            if let Some(value) = value {
                postings
                    .entry((*value).to_owned())
                    .or_insert_with(roaring::RoaringBitmap::new)
                    .insert(id as u32);
            }
        }
        write_keyword_segment(path, 4, values, &postings).unwrap();
        Arc::new(SegmentReader::open(path).unwrap())
    }

    fn set_source(path: &Path, rows: &[Option<Vec<String>>]) -> Arc<SegmentReader> {
        let mut postings = BTreeMap::new();
        for (id, row) in rows.iter().enumerate() {
            for member in row.as_deref().unwrap_or_default() {
                postings
                    .entry(member.clone())
                    .or_insert_with(roaring::RoaringBitmap::new)
                    .insert(id as u32);
            }
        }
        let refs: Vec<Option<&[String]>> = rows.iter().map(Option::as_deref).collect();
        write_set_segment(path, 4, &refs, &postings).unwrap();
        Arc::new(SegmentReader::open(path).unwrap())
    }

    struct Projection {
        rows: Vec<Option<u32>>,
        terms: Vec<(String, Arc<(Vec<u32>, Vec<u32>)>)>,
    }

    impl TextStreamView for Projection {
        fn n_docs(&self) -> u32 {
            self.rows.len() as u32
        }

        fn text_is_present(&self, id: u32) -> bool {
            self.rows.get(id as usize).is_some_and(Option::is_some)
        }

        fn text_doc_len(&self, id: u32) -> u32 {
            self.rows.get(id as usize).and_then(|row| *row).unwrap_or(0)
        }

        fn terms<'a>(&'a self) -> Result<Box<dyn Iterator<Item = Result<Cow<'a, str>>> + 'a>> {
            Ok(Box::new(
                self.terms
                    .iter()
                    .map(|(term, _)| Ok(Cow::Borrowed(term.as_str()))),
            ))
        }

        fn text_postings(&self, term: &str) -> Result<Option<Arc<(Vec<u32>, Vec<u32>)>>> {
            Ok(self
                .terms
                .iter()
                .find(|(candidate, _)| candidate == term)
                .map(|(_, posting)| posting.clone()))
        }
    }

    #[test]
    fn text_projection_lends_a_large_raw_term_through_composed_second_checkpoint() {
        let dir = tempfile::tempdir().unwrap();
        let large = "x".repeat(VAR_BLOCK_BYTES + 1);
        let base_path = dir.path().join("raw-text-base.lseg");
        let base_view = Projection {
            rows: vec![Some(3)],
            terms: vec![(large.clone(), Arc::new((vec![0], vec![3])))],
        };
        write_text_projection(&base_path, 70, &base_view).unwrap();
        let base = Arc::new(SegmentReader::open(&base_path).unwrap());
        assert!(matches!(
            base.keyword_term_at_ordinal_cow(0),
            Some(Cow::Borrowed(term)) if term.len() == large.len()
        ));

        let composed = ComposedSegmentReader::from_base(base);
        let mut terms = TextStreamView::terms(&composed).unwrap();
        let term: Cow<'_, str> = terms.next().unwrap().unwrap().into();
        assert!(matches!(term, Cow::Borrowed(term) if term.len() == large.len()));
        assert!(terms.next().is_none());

        let target = dir.path().join("raw-text-second.lseg");
        write_text_stream(&target, 71, &composed).unwrap();
        let reader = SegmentReader::open(&target).unwrap();
        assert!(matches!(
            reader.keyword_term_at_ordinal_cow(0),
            Some(Cow::Borrowed(term)) if term.len() == large.len()
        ));
        assert_eq!(reader.text_postings(&large), Some((vec![0], vec![3])));
        assert_eq!(reader.text_doc_len(0), 3);
        assert_eq!(reader.text_doc_count(), 1);
        assert_eq!(reader.text_total_doc_len(), 3);
    }

    #[test]
    fn stream_text_projection_preserves_sparse_empty_deleted_rows_and_tf() {
        let dir = tempfile::tempdir().unwrap();
        let view = Projection {
            // Row 1 is deleted/absent. Row 2 is explicitly empty.
            rows: vec![Some(3), None, Some(0), Some(2)],
            terms: vec![
                ("alpha".into(), Arc::new((vec![0, 3], vec![2, 1]))),
                ("zeta".into(), Arc::new((vec![0], vec![1]))),
            ],
        };
        let path = dir.path().join("projection.lseg");
        write_text_projection(&path, 17, &view).unwrap();
        let reader = SegmentReader::open(&path).unwrap();
        assert_eq!(reader.applied_seq(), 17);
        assert!(reader.text_is_present(0));
        assert!(!reader.text_is_present(1));
        assert!(reader.text_is_present(2));
        assert!(reader.text_is_present(3));
        assert_eq!(reader.text_doc_len(2), 0);
        assert_eq!(reader.text_doc_count(), 3);
        assert_eq!(reader.text_total_doc_len(), 5);
        assert_eq!(
            reader.text_postings("alpha"),
            Some((vec![0, 3], vec![2, 1]))
        );
        assert_eq!(reader.text_postings("zeta"), Some((vec![0], vec![1])));
    }

    #[test]
    fn stream_text_writes_normal_terms_and_rows() {
        let dir = tempfile::tempdir().unwrap();
        let base = source(
            &dir.path().join("base"),
            &[Some(&[("ant", 2)]), Some(&[("bee", 1)])],
        );
        let path = dir.path().join("stream");
        write_text_stream(&path, 9, &ComposedSegmentReader::from_base(base)).unwrap();
        let reader = SegmentReader::open(&path).unwrap();
        assert_eq!(reader.applied_seq(), 9);
        assert_eq!(reader.text_postings("ant"), Some((vec![0], vec![2])));
        assert_eq!(reader.text_postings("bee"), Some((vec![1], vec![1])));
        assert_eq!(reader.text_doc_count(), 2);
        assert_eq!(reader.text_total_doc_len(), 3);
    }

    #[test]
    fn stream_text_keeps_empty_present_distinct_from_absent() {
        let dir = tempfile::tempdir().unwrap();
        let base = source(&dir.path().join("base"), &[None, Some(&[])]);
        let path = dir.path().join("stream");
        write_text_stream(&path, 1, &ComposedSegmentReader::from_base(base)).unwrap();
        let reader = SegmentReader::open(&path).unwrap();
        assert!(!reader.text_is_present(0));
        assert!(reader.text_is_present(1));
        assert_eq!(reader.text_doc_len(0), 0);
        assert_eq!(reader.text_doc_len(1), 0);
        assert_eq!(reader.text_doc_count(), 1);
        assert_eq!(reader.text_total_doc_len(), 0);
        assert_eq!(reader.text_postings("missing"), None);
    }

    #[test]
    fn stream_text_single_row_aligns_present_bitset() {
        let dir = tempfile::tempdir().unwrap();
        let base = source(&dir.path().join("base"), &[Some(&[])]);
        let path = dir.path().join("stream");
        write_text_stream(&path, 2, &ComposedSegmentReader::from_base(base)).unwrap();
        let reader = SegmentReader::open(&path).unwrap();
        assert_eq!(reader.n_docs(), 1);
        assert!(reader.text_is_present(0));
        assert_eq!(reader.text_doc_len(0), 0);
        assert_eq!(reader.text_doc_count(), 1);
        assert_eq!(reader.text_total_doc_len(), 0);
    }

    #[test]
    fn stream_text_zero_rows_has_empty_fixed_columns_and_corpus() {
        let dir = tempfile::tempdir().unwrap();
        let base = source(&dir.path().join("base"), &[]);
        let path = dir.path().join("stream");
        write_text_stream(&path, 3, &ComposedSegmentReader::from_base(base)).unwrap();
        let reader = SegmentReader::open(&path).unwrap();
        assert_eq!(reader.n_docs(), 0);
        assert!(!reader.text_is_present(0));
        assert_eq!(reader.text_doc_len(0), 0);
        assert_eq!(reader.text_doc_count(), 0);
        assert_eq!(reader.text_total_doc_len(), 0);
    }

    #[test]
    fn stream_text_resolves_sparse_delta_before_seal() {
        let dir = tempfile::tempdir().unwrap();
        let base = source(
            &dir.path().join("base"),
            &[Some(&[("old", 1)]), Some(&[("stay", 2)])],
        );
        let delta = source(
            &dir.path().join("delta"),
            &[Some(&[("new", 3)]), None, Some(&[])],
        );
        let view = ComposedSegmentReader::from_base(base)
            .with_delta(delta, vec![0, 1, 4])
            .unwrap();
        let path = dir.path().join("stream");
        write_text_stream(&path, 11, &view).unwrap();
        let reader = SegmentReader::open(&path).unwrap();
        assert_eq!(reader.n_docs(), 5);
        assert_eq!(reader.text_postings("old"), None);
        assert_eq!(reader.text_postings("stay"), None);
        assert_eq!(reader.text_postings("new"), Some((vec![0], vec![3])));
        assert!(reader.text_is_present(0));
        assert!(!reader.text_is_present(1));
        assert!(!reader.text_is_present(2));
        assert!(reader.text_is_present(4));
        assert_eq!(reader.text_doc_len(4), 0);
        assert_eq!(reader.text_doc_count(), 2);
        assert_eq!(reader.text_total_doc_len(), 3);
    }

    #[test]
    fn stream_keyword_resolves_layered_update_delete_and_empty_odd_rows() {
        let dir = tempfile::tempdir().unwrap();
        let base = keyword_source(&dir.path().join("base"), &[Some("old"), Some("keep"), None]);
        let delta = keyword_source(&dir.path().join("delta"), &[Some("new"), None, Some("")]);
        let view = ComposedSegmentReader::from_base(base)
            .with_delta(delta, vec![0, 1, 4])
            .unwrap();
        let path = dir.path().join("stream");
        write_keyword_stream(&path, 8, &view).unwrap();
        let reader = SegmentReader::open(&path).unwrap();
        assert_eq!(reader.n_docs(), 5);
        assert_eq!(reader.keyword_at(0), Some("new".to_owned()));
        assert_eq!(reader.keyword_at(1), None);
        assert_eq!(reader.keyword_at(4), Some(String::new()));
        assert_eq!(reader.keyword_postings("old"), None);
        assert_eq!(
            reader
                .keyword_postings("new")
                .unwrap()
                .iter()
                .collect::<Vec<_>>(),
            vec![0]
        );
    }

    #[test]
    fn stream_set_resolves_layered_delete_and_explicit_empty_odd_rows() {
        let dir = tempfile::tempdir().unwrap();
        let base = set_source(
            &dir.path().join("base"),
            &[Some(vec!["old".to_owned()]), Some(Vec::new()), None],
        );
        let delta = set_source(
            &dir.path().join("delta"),
            &[None, Some(vec!["new".to_owned()]), Some(Vec::new())],
        );
        let view = ComposedSegmentReader::from_base(base)
            .with_delta(delta, vec![0, 1, 4])
            .unwrap();
        let path = dir.path().join("stream");
        write_set_stream(&path, 8, &view).unwrap();
        let reader = SegmentReader::open(&path).unwrap();
        assert_eq!(reader.n_docs(), 5);
        assert_eq!(reader.set_at(0), None);
        assert_eq!(reader.set_at(1), Some(vec!["new".to_owned()]));
        assert_eq!(reader.set_at(4), Some(Vec::new()));
        assert_eq!(reader.set_postings("old"), None);
        assert_eq!(
            reader
                .set_postings("new")
                .unwrap()
                .iter()
                .collect::<Vec<_>>(),
            vec![1]
        );
    }

    #[test]
    fn stream_number_resolves_layered_updates_and_sorted_postings() {
        let dir = tempfile::tempdir().unwrap();
        let base_path = dir.path().join("base");
        write_number_segment(&base_path, 4, &[Some(-2.0), Some(4.0), None]).unwrap();
        let delta_path = dir.path().join("delta");
        write_number_segment(&delta_path, 5, &[Some(3.0), None, Some(-1.0)]).unwrap();
        let view =
            ComposedSegmentReader::from_base(Arc::new(SegmentReader::open(&base_path).unwrap()))
                .with_delta(
                    Arc::new(SegmentReader::open(&delta_path).unwrap()),
                    vec![0, 1, 4],
                )
                .unwrap();
        let path = dir.path().join("stream");
        write_number_stream(&path, 8, &view).unwrap();
        let reader = SegmentReader::open(&path).unwrap();
        assert_eq!(reader.n_docs(), 5);
        assert_eq!(reader.number_at(0), Some(3.0));
        assert_eq!(reader.number_at(1), None);
        assert_eq!(reader.number_at(4), Some(-1.0));
        assert_eq!(
            reader
                .number_value_postings(sortable_bits(-1.0))
                .unwrap()
                .iter()
                .collect::<Vec<_>>(),
            vec![4]
        );
        assert!(reader.number_value_postings(sortable_bits(4.0)).is_none());
    }

    #[test]
    fn stream_hash_resolves_zero_delete_and_odd_rows() {
        let dir = tempfile::tempdir().unwrap();
        let base_path = dir.path().join("base");
        write_hash_segment(&base_path, 4, &[Some(7), Some(0), None]).unwrap();
        let delta_path = dir.path().join("delta");
        write_hash_segment(&delta_path, 5, &[None, Some(9), Some(0)]).unwrap();
        let view =
            ComposedSegmentReader::from_base(Arc::new(SegmentReader::open(&base_path).unwrap()))
                .with_delta(
                    Arc::new(SegmentReader::open(&delta_path).unwrap()),
                    vec![0, 1, 4],
                )
                .unwrap();
        let path = dir.path().join("stream");
        write_hash_stream(&path, 8, &view).unwrap();
        let reader = SegmentReader::open(&path).unwrap();
        assert_eq!(reader.n_docs(), 5);
        assert_eq!(reader.hash_at(0), None);
        assert_eq!(reader.hash_at(1), Some(9));
        assert_eq!(reader.hash_at(4), Some(0));
    }

    #[test]
    fn stream_keyword_large_dictionary_uses_one_binary_lookup_per_row() {
        let dir = tempfile::tempdir().unwrap();
        let values: Vec<String> = (0..257).map(|id| format!("term-{id:04}")).collect();
        let rows: Vec<Option<&str>> = values.iter().map(|value| Some(value.as_str())).collect();
        let base = keyword_source(&dir.path().join("base"), &rows);
        let path = dir.path().join("stream");
        STREAM_SPOOL_LOOKUPS.with(|lookups| lookups.set(0));
        write_keyword_stream(&path, 9, &ComposedSegmentReader::from_base(base)).unwrap();
        let reader = SegmentReader::open(&path).unwrap();
        assert_eq!(reader.keyword_at(0), Some("term-0000".to_owned()));
        assert_eq!(reader.keyword_at(256), Some("term-0256".to_owned()));
        // The spool has 257 distinct terms, but every forward row calls exactly
        // one SegmentReader binary lookup.  It never performs a term walk.
        STREAM_SPOOL_LOOKUPS.with(|lookups| assert_eq!(lookups.get(), 257));
    }

    #[test]
    fn dictionary_spool_removes_only_its_created_temp() {
        let dir = tempfile::tempdir().unwrap();
        let base = keyword_source(&dir.path().join("base"), &[Some("one")]);
        let view = ComposedSegmentReader::from_base(base);
        let spool = DictionarySpool::build(&dir.path().join("target"), 9, &view, |term| {
            view.keyword_postings(term).is_some()
        })
        .unwrap();
        let path = spool.path.clone();
        assert!(path.exists());
        drop(spool);
        assert!(!path.exists());
    }

    #[test]
    fn stream_vector_round_trips_none_zero_vector_and_odd_rows() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("stream");
        write_vector_stream(&path, 12, 3, 2, |id| {
            Ok(match id {
                0 => Some(vec![1.0, -2.0]),
                1 => None,
                2 => Some(vec![0.0, 0.0]),
                _ => unreachable!(),
            })
        })
        .unwrap();
        let reader = SegmentReader::open(&path).unwrap();
        assert_eq!(reader.n_docs(), 3);
        assert_eq!(reader.vector_at(0, 2), Some(&[1.0, -2.0][..]));
        assert_eq!(reader.vector_at(1, 2), None);
        assert_eq!(reader.vector_at(2, 2), Some(&[0.0, 0.0][..]));
    }

    #[test]
    fn stream_vector_zero_rows_does_not_call_callback() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("stream");
        let calls = std::cell::Cell::new(0);
        write_vector_stream(&path, 12, 0, 2, |_| {
            calls.set(calls.get() + 1);
            Ok(Some(vec![1.0, 2.0]))
        })
        .unwrap();
        let reader = SegmentReader::open(&path).unwrap();
        assert_eq!(calls.get(), 0);
        assert_eq!(reader.n_docs(), 0);
        assert_eq!(reader.vectors_slice(2), Some(&[][..]));
    }

    #[test]
    fn stream_vector_bad_row_removes_its_temp_and_target() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("target");
        assert!(write_vector_stream(&path, 12, 1, 2, |_| Ok(Some(vec![1.0]))).is_err());
        assert!(!path.exists());
        assert!(std::fs::read_dir(dir.path()).unwrap().all(|entry| !entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .contains("target.stream")));
        assert!(write_vector_stream(&path, 12, 1, 2, |_| Ok(Some(vec![f32::NAN, 1.0]))).is_err());
        assert!(!path.exists());
    }

    #[test]
    fn scalar_projection_keyword_matches_normal_writer_for_sparse_duplicates_and_large_term() {
        let dir = tempfile::tempdir().unwrap();
        let large = "x".repeat(VAR_BLOCK_BYTES + 17);
        let rows = vec![
            Some("z"),
            None,
            Some(large.as_str()),
            Some("a"),
            Some(large.as_str()),
        ];
        let base = keyword_source(&dir.path().join("base"), &rows);
        let view = ComposedSegmentReader::from_base(base.clone());
        let ordinary = dir.path().join("ordinary");
        let projected = dir.path().join("projected");
        write_keyword_stream(&ordinary, 31, &view).unwrap();
        write_keyword_projection(
            &projected,
            31,
            &BorrowedKeywordRows(&rows),
            ScalarProjectionScratch::new(usize::MAX),
        )
        .unwrap();
        let left = SegmentReader::open(&ordinary).unwrap();
        let right = SegmentReader::open(&projected).unwrap();
        for id in 0..5 {
            assert_eq!(left.keyword_at(id), right.keyword_at(id));
        }
        for term in ["a", "z", large.as_str()] {
            assert_eq!(left.keyword_postings(term), right.keyword_postings(term));
        }
    }

    #[test]
    fn scalar_projection_set_matches_normal_writer_for_sparse_empty_duplicates_and_sorted_terms() {
        let dir = tempfile::tempdir().unwrap();
        let large = "q".repeat(VAR_BLOCK_BYTES + 3);
        let rows = vec![
            Some(vec!["a".to_owned(), "z".to_owned()]),
            None,
            Some(Vec::new()),
            Some(vec!["a".to_owned(), large.clone()]),
        ];
        let base = set_source(&dir.path().join("base"), &rows);
        let view = ComposedSegmentReader::from_base(base);
        let ordinary = dir.path().join("ordinary");
        let projected = dir.path().join("projected");
        write_set_stream(&ordinary, 32, &view).unwrap();
        write_set_projection(
            &projected,
            32,
            &view,
            ScalarProjectionScratch::new(usize::MAX),
        )
        .unwrap();
        let left = SegmentReader::open(&ordinary).unwrap();
        let right = SegmentReader::open(&projected).unwrap();
        for id in 0..4 {
            assert_eq!(left.set_at(id), right.set_at(id));
        }
        for term in ["a", "z", large.as_str()] {
            assert_eq!(left.set_postings(term), right.set_postings(term));
        }
    }

    #[test]
    fn scalar_projection_number_matches_normal_writer_for_sparse_and_sorted_keys() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("base");
        write_number_segment(
            &source,
            4,
            &[Some(-3.0), None, Some(0.0), Some(-3.0), Some(8.5)],
        )
        .unwrap();
        let view =
            ComposedSegmentReader::from_base(Arc::new(SegmentReader::open(&source).unwrap()));
        let ordinary = dir.path().join("ordinary");
        let projected = dir.path().join("projected");
        write_number_stream(&ordinary, 33, &view).unwrap();
        write_number_projection(
            &projected,
            33,
            &view,
            ScalarProjectionScratch::new(usize::MAX),
        )
        .unwrap();
        let left = SegmentReader::open(&ordinary).unwrap();
        let right = SegmentReader::open(&projected).unwrap();
        for id in 0..5 {
            assert_eq!(left.number_at(id), right.number_at(id));
        }
        for value in [-3.0, 0.0, 8.5] {
            assert_eq!(
                left.number_value_postings(sortable_bits(value)),
                right.number_value_postings(sortable_bits(value))
            );
        }
    }

    #[test]
    fn scalar_projection_refuses_uncharged_var_entry_before_target_visibility() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("target");
        let large = "x".repeat(VAR_BLOCK_BYTES + 17);
        let error = projection_dictionary(
            &target,
            34,
            ScalarProjectionScratch::new(VAR_BLOCK_BYTES),
            |emit| emit(&large),
        )
        .err()
        .expect("unreserved dictionary term must fail before publishing a target");
        let shortage = error
            .downcast_ref::<ScalarProjectionScratchRequired>()
            .unwrap();
        assert_eq!(shortage.kind, "dictionary term");
        assert_eq!(shortage.required, large.len());
        assert!(!target.exists());
    }

    #[test]
    fn scalar_projection_refuses_changed_posting_replay() {
        let mut pass = false;
        let error = projection_posting(
            3,
            ScalarProjectionScratch::new(32),
            "keyword posting",
            |emit| {
                if pass {
                    emit(1)?;
                    emit(2)?;
                } else {
                    pass = true;
                    emit(1)?;
                }
                Ok(true)
            },
        )
        .unwrap_err();
        assert!(error
            .to_string()
            .contains("changed between rewindable posting passes"));
    }
    #[test]
    fn scalar_projection_refuses_same_size_changed_posting_replay() {
        let mut replay = false;
        let result = projection_posting(
            4,
            ScalarProjectionScratch::new(32),
            "keyword posting",
            |emit| {
                emit(if replay { 2 } else { 1 })?;
                replay = true;
                Ok(true)
            },
        );
        assert!(
            result.is_err(),
            "same-size posting replay must keep the exact row identity"
        );
    }
    #[test]
    fn scalar_projection_shared_prefix_cannot_expand_a_whole_dictionary_block() {
        let root = tempfile::tempdir().unwrap();
        let terms: Vec<_> = (0..32)
            .map(|n| format!("{}-{n:04}", "x".repeat(8192)))
            .collect();
        let max = terms.iter().map(String::len).max().unwrap();
        let target = root.path().join("projection.lseg");
        let spool = projection_dictionary(&target, 1, ScalarProjectionScratch::new(max), |emit| {
            for term in &terms {
                emit(term)?;
            }
            Ok(())
        })
        .unwrap();
        let (first_block, _) = spool.reader.dict_block_at(ROLE_DICT, 0).unwrap();
        let full_bytes: usize = first_block.iter().map(Vec::len).sum();
        assert!(
            full_bytes <= VAR_BLOCK_BYTES + max,
            "projection prefix decoding must stay within one full-entry block"
        );
        assert!(first_block.len() < terms.len());
        for (ordinal, term) in terms.iter().enumerate() {
            assert_eq!(spool.dict_id(term).unwrap(), ordinal as u32);
        }
    }

    #[test]
    fn raw_scalar_dictionary_keyword_borrows_empty_and_large_terms() {
        let root = tempfile::tempdir().unwrap();
        let large = "x".repeat(256 * 1024);
        let rows = vec![Some(""), Some(large.as_str()), None, Some("a")];
        let target = root.path().join("raw-keyword.lseg");
        write_keyword_projection(
            &target,
            71,
            &BorrowedKeywordRows(&rows),
            ScalarProjectionScratch::new(64).raw_scalar_dictionary(),
        )
        .unwrap();
        let reader = SegmentReader::open(&target).unwrap();
        assert_eq!(reader.keyword_at(0).as_deref(), Some(""));
        assert_eq!(reader.keyword_at(1).as_deref(), Some(large.as_str()));
        assert_eq!(reader.keyword_postings(""), Some([0].into_iter().collect()));
        assert!(matches!(
            reader.dict_value(0),
            Some(std::borrow::Cow::Borrowed(""))
        ));
        assert!(matches!(
            reader.dict_value(1),
            Some(std::borrow::Cow::Borrowed(_))
        ));
        assert!(matches!(
            reader.keyword_at_cow(1),
            Some(std::borrow::Cow::Borrowed(_))
        ));
        assert!(
            matches!(
                reader.keyword_term_at_ordinal_cow(1),
                Some(std::borrow::Cow::Borrowed(_))
            ),
            "the ordinal stream must lend the large mmap term"
        );
    }

    #[test]
    fn raw_scalar_dictionary_set_keeps_ordinal_postings() {
        let root = tempfile::tempdir().unwrap();
        let rows = vec![
            Some(vec!["".to_owned(), "b".to_owned()]),
            Some(vec!["a".to_owned()]),
            None,
        ];
        let base = set_source(&root.path().join("base"), &rows);
        let target = root.path().join("raw-set.lseg");
        write_set_projection(
            &target,
            72,
            &ComposedSegmentReader::from_base(base),
            ScalarProjectionScratch::new(64).raw_scalar_dictionary(),
        )
        .unwrap();
        let reader = SegmentReader::open(&target).unwrap();
        assert_eq!(reader.set_at(0), Some(vec!["".to_owned(), "b".to_owned()]));
        assert_eq!(reader.set_postings("a"), Some([1].into_iter().collect()));
        assert!(matches!(
            reader.dict_value(0),
            Some(std::borrow::Cow::Borrowed(""))
        ));
        assert!(matches!(
            reader.set_member_at_cow(0, 0),
            Some(std::borrow::Cow::Borrowed(""))
        ));
    }

    #[test]
    fn raw_scalar_dictionary_refuses_bad_offsets_utf8_and_unknown_codec() {
        let root = tempfile::tempdir().unwrap();
        let target = root.path().join("raw-invalid.lseg");
        let rows = vec![Some("a")];
        write_keyword_projection(
            &target,
            73,
            &BorrowedKeywordRows(&rows),
            ScalarProjectionScratch::new(64).raw_scalar_dictionary(),
        )
        .unwrap();
        let bytes = std::fs::read(&target).unwrap();
        let footer = Footer::from_bytes(&bytes[bytes.len() - FOOTER_LEN..]).unwrap();
        let start = footer.dir_offset as usize;
        let end = start + footer.dir_len as usize;
        let mut dir: Vec<ColumnRef> = ciborium::from_reader(&bytes[start..end]).unwrap();
        let offsets = dir
            .iter()
            .find(|c| c.role == ROLE_DICT_OFFSETS)
            .unwrap()
            .clone();
        let mut file = OpenOptions::new().write(true).open(&target).unwrap();
        file.seek(SeekFrom::Start(offsets.byte_offset + 8)).unwrap();
        file.write_all(&99u64.to_le_bytes()).unwrap();
        assert!(SegmentReader::open(&target)
            .unwrap_err()
            .to_string()
            .contains("offset"));
        // Restore a valid file, then poison raw bytes without changing its directory.
        write_keyword_projection(
            &target,
            73,
            &BorrowedKeywordRows(&rows),
            ScalarProjectionScratch::new(64).raw_scalar_dictionary(),
        )
        .unwrap();
        let dict = dir.iter().find(|c| c.role == ROLE_DICT).unwrap().clone();
        let mut file = OpenOptions::new().write(true).open(&target).unwrap();
        file.seek(SeekFrom::Start(dict.byte_offset)).unwrap();
        file.write_all(&[0xff]).unwrap();
        assert!(SegmentReader::open(&target)
            .unwrap_err()
            .to_string()
            .contains("UTF-8"));
        // Change only the CBOR directory codec byte and refresh its footer CRC.
        write_keyword_projection(
            &target,
            73,
            &BorrowedKeywordRows(&rows),
            ScalarProjectionScratch::new(64).raw_scalar_dictionary(),
        )
        .unwrap();
        let bytes = std::fs::read(&target).unwrap();
        let footer = Footer::from_bytes(&bytes[bytes.len() - FOOTER_LEN..]).unwrap();
        let start = footer.dir_offset as usize;
        let end = start + footer.dir_len as usize;
        let mut dir: Vec<ColumnRef> = ciborium::from_reader(&bytes[start..end]).unwrap();
        dir.iter_mut().find(|c| c.role == ROLE_DICT).unwrap().codec = 3;
        let mut encoded = Vec::new();
        ciborium::into_writer(&dir, &mut encoded).unwrap();
        assert_eq!(encoded.len(), footer.dir_len as usize);
        let mut changed = bytes;
        changed[start..end].copy_from_slice(&encoded);
        let mut footer = footer;
        footer.crc32 = crc32fast::hash(&encoded);
        let tail = changed.len() - FOOTER_LEN;
        changed[tail..].copy_from_slice(&footer.to_bytes());
        std::fs::write(&target, changed).unwrap();
        assert!(SegmentReader::open(&target)
            .unwrap_err()
            .to_string()
            .contains("unsupported segment column codec"));
    }

    fn rewrite_directory(path: &Path, change: impl FnOnce(&mut Vec<ColumnRef>)) {
        let bytes = std::fs::read(path).unwrap();
        let mut footer = Footer::from_bytes(&bytes[bytes.len() - FOOTER_LEN..]).unwrap();
        let start = footer.dir_offset as usize;
        let end = start + footer.dir_len as usize;
        let mut dir: Vec<ColumnRef> = ciborium::from_reader(&bytes[start..end]).unwrap();
        change(&mut dir);
        let mut encoded = Vec::new();
        ciborium::into_writer(&dir, &mut encoded).unwrap();
        footer.dir_len = encoded.len() as u64;
        footer.crc32 = crc32fast::hash(&encoded);
        let mut changed = bytes[..start].to_vec();
        changed.extend_from_slice(&encoded);
        changed.extend_from_slice(&footer.to_bytes());
        std::fs::write(path, changed).unwrap();
    }

    #[test]
    fn raw_scalar_dictionary_refuses_split_codepoint_unsorted_and_duplicate_entries() {
        let root = tempfile::tempdir().unwrap();
        let target = root.path().join("raw-entry-validity.lseg");
        let rows = vec![Some("a"), Some("é"), Some("z")];
        write_keyword_projection(
            &target,
            74,
            &BorrowedKeywordRows(&rows),
            ScalarProjectionScratch::new(64).raw_scalar_dictionary(),
        )
        .unwrap();
        let bytes = std::fs::read(&target).unwrap();
        let footer = Footer::from_bytes(&bytes[bytes.len() - FOOTER_LEN..]).unwrap();
        let dir: Vec<ColumnRef> = ciborium::from_reader(
            &bytes[footer.dir_offset as usize..(footer.dir_offset + footer.dir_len) as usize],
        )
        .unwrap();
        let offsets = dir
            .iter()
            .find(|column| column.role == ROLE_DICT_OFFSETS)
            .unwrap();
        let mut file = OpenOptions::new().write(true).open(&target).unwrap();
        // Split the two-byte UTF-8 `é` between two dictionary entries while
        // retaining a monotone table that still ends at the data length.
        file.seek(SeekFrom::Start(offsets.byte_offset + 16))
            .unwrap();
        file.write_all(&3u64.to_le_bytes()).unwrap();
        assert!(SegmentReader::open(&target)
            .unwrap_err()
            .to_string()
            .contains("entry is not UTF-8"));

        let sorted = vec![Some("a"), Some("b")];
        write_keyword_projection(
            &target,
            74,
            &BorrowedKeywordRows(&sorted),
            ScalarProjectionScratch::new(64).raw_scalar_dictionary(),
        )
        .unwrap();
        let bytes = std::fs::read(&target).unwrap();
        let footer = Footer::from_bytes(&bytes[bytes.len() - FOOTER_LEN..]).unwrap();
        let dir: Vec<ColumnRef> = ciborium::from_reader(
            &bytes[footer.dir_offset as usize..(footer.dir_offset + footer.dir_len) as usize],
        )
        .unwrap();
        let dict = dir.iter().find(|column| column.role == ROLE_DICT).unwrap();
        let mut file = OpenOptions::new().write(true).open(&target).unwrap();
        file.seek(SeekFrom::Start(dict.byte_offset)).unwrap();
        file.write_all(b"ba").unwrap();
        assert!(SegmentReader::open(&target)
            .unwrap_err()
            .to_string()
            .contains("strictly sorted"));

        write_keyword_projection(
            &target,
            74,
            &BorrowedKeywordRows(&sorted),
            ScalarProjectionScratch::new(64).raw_scalar_dictionary(),
        )
        .unwrap();
        let mut file = OpenOptions::new().write(true).open(&target).unwrap();
        file.seek(SeekFrom::Start(dict.byte_offset)).unwrap();
        file.write_all(b"aa").unwrap();
        assert!(SegmentReader::open(&target)
            .unwrap_err()
            .to_string()
            .contains("strictly sorted"));
    }

    #[test]
    fn raw_scalar_dictionary_refuses_duplicate_and_orphan_offset_roles() {
        let root = tempfile::tempdir().unwrap();
        let target = root.path().join("raw-roles.lseg");
        let rows = vec![Some("a")];
        let write = || {
            write_keyword_projection(
                &target,
                75,
                &BorrowedKeywordRows(&rows),
                ScalarProjectionScratch::new(64).raw_scalar_dictionary(),
            )
            .unwrap()
        };
        write();
        rewrite_directory(&target, |dir| {
            let dict = dir
                .iter()
                .find(|column| column.role == ROLE_DICT)
                .unwrap()
                .clone();
            dir.push(dict);
        });
        assert!(SegmentReader::open(&target)
            .unwrap_err()
            .to_string()
            .contains("duplicate dictionary"));
        write();
        rewrite_directory(&target, |dir| dir.retain(|column| column.role != ROLE_DICT));
        assert!(SegmentReader::open(&target)
            .unwrap_err()
            .to_string()
            .contains("orphan raw dictionary offsets"));
        write();
        rewrite_directory(&target, |dir| {
            let offsets = dir
                .iter()
                .find(|column| column.role == ROLE_DICT_OFFSETS)
                .unwrap()
                .clone();
            dir.push(offsets);
        });
        assert!(SegmentReader::open(&target)
            .unwrap_err()
            .to_string()
            .contains("duplicate raw dictionary offsets"));
    }

    #[test]
    fn composed_raw_keyword_projection_lends_large_row_and_term() {
        let root = tempfile::tempdir().unwrap();
        let large = "q".repeat(256 * 1024);
        let rows = vec![Some("a"), Some(large.as_str()), None];
        let source = root.path().join("raw-composed-keyword-source");
        write_keyword_projection(
            &source,
            81,
            &BorrowedKeywordRows(&rows),
            ScalarProjectionScratch::new(64).raw_scalar_dictionary(),
        )
        .unwrap();
        let view =
            ComposedSegmentReader::from_base(Arc::new(SegmentReader::open(&source).unwrap()));
        assert!(
            matches!(view.keyword_at_cow(1), Some(std::borrow::Cow::Borrowed(value)) if value.len() == large.len())
        );
        let mut terms = view.string_terms(false).unwrap();
        assert!(matches!(
            terms.next_cow().unwrap(),
            Some(std::borrow::Cow::Borrowed("a"))
        ));
        assert!(
            matches!(terms.next_cow().unwrap(), Some(std::borrow::Cow::Borrowed(value)) if value.len() == large.len())
        );
        let target = root.path().join("raw-composed-keyword-target");
        write_keyword_projection(
            &target,
            82,
            &view,
            ScalarProjectionScratch::new(64).raw_scalar_dictionary(),
        )
        .unwrap();
        let reader = SegmentReader::open(&target).unwrap();
        assert_eq!(reader.keyword_at(1).as_deref(), Some(large.as_str()));
        assert_eq!(
            reader.keyword_postings(large.as_str()),
            Some([1].into_iter().collect())
        );
    }

    #[test]
    fn composed_raw_set_projection_lends_members_and_accepts_legacy_source() {
        let root = tempfile::tempdir().unwrap();
        let large = "z".repeat(256 * 1024);
        let rows = vec![
            Some(vec!["a".to_owned(), large.clone()]),
            Some(Vec::new()),
            None,
        ];
        let legacy = set_source(&root.path().join("legacy-set"), &rows);
        let legacy_view = ComposedSegmentReader::from_base(legacy);
        // LZ4 is a one-entry owned fallback, yet its output may select raw.
        assert!(matches!(
            legacy_view.set_member_at_cow(0, 1),
            Some(std::borrow::Cow::Owned(_))
        ));
        let source = root.path().join("raw-composed-set-source");
        write_set_projection(
            &source,
            83,
            &legacy_view,
            ScalarProjectionScratch::new(64).raw_scalar_dictionary(),
        )
        .unwrap();
        let raw_view =
            ComposedSegmentReader::from_base(Arc::new(SegmentReader::open(&source).unwrap()));
        assert!(
            matches!(raw_view.set_member_at_cow(0, 1), Some(std::borrow::Cow::Borrowed(value)) if value.len() == large.len())
        );
        let target = root.path().join("raw-composed-set-target");
        write_set_projection(
            &target,
            84,
            &raw_view,
            ScalarProjectionScratch::new(64).raw_scalar_dictionary(),
        )
        .unwrap();
        let reader = SegmentReader::open(&target).unwrap();
        assert_eq!(reader.set_at(0), Some(vec!["a".to_owned(), large.clone()]));
        assert_eq!(
            reader.set_postings(large.as_str()),
            Some([0].into_iter().collect())
        );
    }
}

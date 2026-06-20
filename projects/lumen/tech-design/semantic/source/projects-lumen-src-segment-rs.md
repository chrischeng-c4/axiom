---
id: projects-lumen-src-segment-rs
capability_refs:
  - id: "competitor-feature-parity"
    role: primary
    claim: "query-planner-boolean-eval-roaring-postings"
    coverage: partial
    rationale: "This source unit is captured as a per-file rust-source-unit during lumen td_ast standardization."
fill_sections: [overview, source, changes]
---

# Standardized projects/lumen/src/segment.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/lumen/src/segment.rs` captured as a per-file rust-source-unit (td_ast) during lumen standardization onto the per-file codegen ladder.

### Symbols

| Name | Target | Kind | Visibility |
|------|--------|------|------------|
| `ColumnRef` | projects/lumen/src/segment.rs | struct | pub |
| `write_number_segment` | projects/lumen/src/segment.rs | function | pub |
| `write_hash_segment` | projects/lumen/src/segment.rs | function | pub |
| `write_vector_segment` | projects/lumen/src/segment.rs | function | pub |
| `write_keyword_segment` | projects/lumen/src/segment.rs | function | pub |
| `write_set_segment` | projects/lumen/src/segment.rs | function | pub |
| `write_text_segment` | projects/lumen/src/segment.rs | function | pub |
| `write_eid_segment` | projects/lumen/src/segment.rs | function | pub |
| `SegmentReader` | projects/lumen/src/segment.rs | struct | pub |
| `open` | projects/lumen/src/segment.rs | function | pub |
| `applied_seq` | projects/lumen/src/segment.rs | function | pub |
| `n_docs` | projects/lumen/src/segment.rs | function | pub |
| `number_at` | projects/lumen/src/segment.rs | function | pub |
| `number_distinct_count` | projects/lumen/src/segment.rs | function | pub |
| `number_value_postings` | projects/lumen/src/segment.rs | function | pub |
| `number_value_df` | projects/lumen/src/segment.rs | function | pub |
| `number_range` | projects/lumen/src/segment.rs | function | pub |
| `number_sorted_bits_at` | projects/lumen/src/segment.rs | function | pub |
| `number_sorted_postings_at` | projects/lumen/src/segment.rs | function | pub |
| `number_range_df` | projects/lumen/src/segment.rs | function | pub |
| `number_range_distinct_count` | projects/lumen/src/segment.rs | function | pub |
| `number_range_index_window` | projects/lumen/src/segment.rs | function | pub |
| `number_values_all` | projects/lumen/src/segment.rs | function | pub |
| `hash_at` | projects/lumen/src/segment.rs | function | pub |
| `vector_at` | projects/lumen/src/segment.rs | function | pub |
| `vectors_slice` | projects/lumen/src/segment.rs | function | pub |
| `keyword_at` | projects/lumen/src/segment.rs | function | pub |
| `keyword_postings` | projects/lumen/src/segment.rs | function | pub |
| `keyword_terms_all` | projects/lumen/src/segment.rs | function | pub |
| `keyword_df` | projects/lumen/src/segment.rs | function | pub |
| `set_at` | projects/lumen/src/segment.rs | function | pub |
| `set_postings` | projects/lumen/src/segment.rs | function | pub |
| `set_df` | projects/lumen/src/segment.rs | function | pub |
| `set_elements_all` | projects/lumen/src/segment.rs | function | pub |
| `text_postings` | projects/lumen/src/segment.rs | function | pub |
| `text_postings_arc` | projects/lumen/src/segment.rs | function | pub |
| `text_doc_len` | projects/lumen/src/segment.rs | function | pub |
| `text_doc_lens` | projects/lumen/src/segment.rs | function | pub |
| `text_token_df` | projects/lumen/src/segment.rs | function | pub |
| `text_doc_count` | projects/lumen/src/segment.rs | function | pub |
| `text_total_doc_len` | projects/lumen/src/segment.rs | function | pub |
| `eid_at` | projects/lumen/src/segment.rs | function | pub |
| `eid_count` | projects/lumen/src/segment.rs | function | pub |
| `eids_all` | projects/lumen/src/segment.rs | function | pub |
| `text_tokens_all` | projects/lumen/src/segment.rs | function | pub |

## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Columnar mmap disk segment — Stage 2 disk-tier (Phase 0 + 2a + 2b).
//!
//! One file = one Number column for `n_docs` rows at a single `applied_seq`.
//! The layout is little-endian and read **tail-first**: the 24-byte footer at
//! the very end points back at a CBOR directory, which in turn locates the
//! page-aligned fixed-width columns. Reads are zero-copy (`mmap` +
//! `bytemuck::try_cast_slice`); a torn / truncated / endian-mismatched file is
//! always reported as an error and **never panics** — the caller discards it
//! and replays from the log, there is no in-place recovery.
//!
//! This module is compiled by default; the disk tier is selected at runtime
//! (`--persistence=segment`), while the in-memory serving engine keeps the CBOR
//! RDB as its default.
//!
//! Phase 2c wired the READER's per-doc lookups (`SegmentReader::number_at` /
//! `n_docs`) into the Number PREDICATE path in `storage.rs`, so those are always
//! live. Phase 2h-3 (WS2 BKD) added the Number RANGE
//! index — a fixed-width ascending `u64[distinct]` SORTED-VALUE column
//! (`ROLE_NUMBER_SORTED`) binary-searched by `number_range` + a parallel
//! per-value docid posting column (`ROLE_NUMBER_POSTINGS`) — so range / exact /
//! boolean queries drive straight off the mmap and the in-RAM `values` BTreeMap
//! is dropped at seal. The WRITER (`write_number_segment`) and
//! `SegmentReader::open` / `applied_seq` are driven by the test seal seam and by
//! the runtime segment-persistence path; in the default (CBOR) configuration they
//! are reachable only from that runtime path, so the module silences dead-code
//! there rather than carry unused `pub` plumbing. The read path that serves live
//! queries is fully exercised and is NOT covered by this allow.
#![cfg_attr(not(test), allow(dead_code))]

use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;

use anyhow::{anyhow, bail, Context, Result};
use byteorder::{LittleEndian, ReadBytesExt};
use serde::{Deserialize, Serialize};

/// Header magic, "LSEG" little-endian.
const MAGIC1: u32 = 0x4C53_4547;
/// Footer magic, "GESL" little-endian.
const MAGIC2: u32 = 0x4753_454C;
/// On-disk format version.
const FORMAT_VER: u32 = 1;
/// Endianness stamp written into the header. On read, a mismatch means the
/// file was produced on a host with a different byte order and must be
/// discarded (we do not byte-swap — discard-and-replay is cheaper than a
/// portable codec for a hot column).
const HOST_ENDIAN_MARKER: u32 = 0x0102_0304;
/// Page size we align fixed-width columns to. mmap hands back a page-aligned
/// base pointer, so a page-aligned column offset yields a `u64`-aligned slice.
const PAGE: usize = 4096;
/// Header is a fixed, zero-padded 4096-byte block at the file start.
const HEADER_LEN: usize = 4096;
/// Footer is a fixed 24-byte block at the file end: dir_offset(8) + dir_len(8)
/// + crc32(4) + magic2(4).
const FOOTER_LEN: usize = 24;

/// Column role discriminant, persisted in [`ColumnRef::role`].
const ROLE_NUMBER: u8 = 0;
const ROLE_PRESENT: u8 = 1;
/// Hash forward column (raw `u64` perceptual hash per doc). Phase 2d.
const ROLE_HASH: u8 = 2;
/// Vector forward column (contiguous `f32[n_docs * dim]`). Phase 2d.
const ROLE_VECTOR: u8 = 3;
/// A prefix-compressed, LZ4-blocked, variable-width string dictionary. Stored
/// in the VAR region after the fixed region; located via the per-column
/// skip-index carried in [`ColumnRef::skip_index`]. Phase 2e-A. Shared by the
/// Keyword and Set sealed columns.
const ROLE_DICT: u8 = 4;
/// Keyword forward column (`u32[n_docs]` dict-id per doc). Phase 2e-A. FIXED
/// width — stays zero-copy via `try_cast_slice`. A sentinel of [`DICT_ABSENT`]
/// marks a doc with no keyword (also gated by the present bitset).
const ROLE_KEYWORD_DICTID: u8 = 5;
/// Set CSR offsets column (`u32[n_docs + 1]`). Phase 2e-A. FIXED width. Doc
/// `i`'s member dict-ids are `packed[offsets[i]..offsets[i+1]]`.
const ROLE_SET_OFFSETS: u8 = 6;
/// Set packed member dict-ids column (`u32[total_members]`). Phase 2e-A. FIXED
/// width, indexed through the CSR offsets column.
const ROLE_SET_PACKED: u8 = 7;
/// Text per-token POSTING-BLOCK column (var-width, LZ4-blocked). Phase 2e-B.
/// Parallel to the [`ROLE_DICT`] token dictionary: the entry at dict-id `t` is
/// token `t`'s posting blob (a delta-varint `docid`-gap + `tf` stream — see
/// [`encode_posting_block`]). Text term-frequency is NOT rebuildable, so unlike
/// the Keyword/Set inverted indexes the postings ARE stored on disk.
const ROLE_TEXT_POSTINGS: u8 = 8;
/// Text per-doc length column (`u32[n_docs]`). Phase 2e-B. FIXED width. Doc
/// `i`'s BM25 length is `doclen[i]` (0 == no value for this field). Read
/// zero-copy; reproduces `TextIndex::doc_len(id)` (which is `lens[id]`, else 0).
const ROLE_TEXT_DOCLEN: u8 = 9;
/// Collection-level external-id DICTIONARY-by-position column (var-width,
/// LZ4-blocked). Phase 2f-1. UNLIKE [`ROLE_DICT`] (which is the SORTED distinct
/// dictionary of a Keyword/Set field), this column stores the external_id string
/// of docid `i` at entry position `i` — i.e. it is the interner's `to_eid` Vec
/// laid out densely in docid order `[0..n_docs)`. It makes a sealed collection
/// SELF-DESCRIBING: reopening the collection rebuilds the whole `Interner` by
/// scanning this column, with no CBOR whole-collection snapshot. Written once
/// per collection seal into `<collection>.lmeta.lseg`. The entries are NOT
/// sorted (docid order, not lexical), so the var-column prefix-delta finds
/// `shared == 0` for unrelated eids and LZ4 does the real compression.
const ROLE_EID: u8 = 10;
/// Keyword per-term INVERTED posting-block column (var-width, LZ4-blocked).
/// Phase 2h-1. Parallel to the Keyword [`ROLE_DICT`] string dictionary: the
/// entry at dict-id `t` is term `t`'s sorted-docid posting blob (a delta-varint
/// `docid`-gap stream with NO tf — a Keyword term holds a doc at most once, so
/// the only fact is membership; see [`encode_docid_block`]). Stored ON DISK so a
/// reopen drives Term/Terms/boolean queries straight off the mmap WITHOUT
/// rebuilding the in-RAM `terms: BTreeMap<String, RoaringBitmap>` inverted index.
/// The per-term doc-count (`df`) is the cheap LEB128 count prefix of the blob
/// (mirrors [`SegmentReader::text_token_df`]), keeping the boolean planner's
/// rarest-first clause ordering cheap. Located by the term's dict index via
/// [`SegmentReader::dict_block_at`].
const ROLE_KEYWORD_POSTINGS: u8 = 11;
/// Set per-element INVERTED posting-block column (var-width, LZ4-blocked).
/// Phase 2h-2. The Set analogue of [`ROLE_KEYWORD_POSTINGS`]: parallel to the
/// Set [`ROLE_DICT`] string dictionary, the entry at dict-id `t` is element
/// `t`'s sorted-docid posting blob — the docids whose set CONTAINS element `t`
/// (a delta-varint `docid`-gap stream with NO tf; a Set holds an element at
/// most once per doc, so the only fact is membership; see
/// [`encode_docid_block`]). Stored ON DISK so a reopen drives membership /
/// Terms / boolean queries straight off the mmap WITHOUT rebuilding the in-RAM
/// `elements: BTreeMap<String, RoaringBitmap>` inverted index (which grows
/// O(distinct set elements)). The per-element doc-count (`df`) is the cheap
/// LEB128 count prefix of the blob, keeping the boolean planner's rarest-first
/// clause ordering cheap. Located by the element's dict index via
/// [`SegmentReader::dict_block_at`].
const ROLE_SET_POSTINGS: u8 = 12;
/// Number SORTED-VALUE range index — the ascending DISTINCT `SortableF64` bit
/// keys (Phase 2h-3). FIXED-WIDTH `u64[distinct]` column: entry `i` is the raw
/// `SortableF64.0` of the `i`-th smallest distinct numeric value in the field.
/// The `SortableF64` transform makes UNSIGNED `u64` order == numeric order, so
/// this column is monotonically ascending and zero-copy binary-searchable on the
/// mmap (`try_cast_slice::<u8, u64>`). The bit keys are EXACTLY the in-RAM
/// `NumberIndex.values` BTreeMap keys, so a binary-search over this column with
/// the identical inclusive/exclusive bounds reproduces `values.range(..)`
/// byte-for-byte. Parallel to [`ROLE_NUMBER_POSTINGS`] (value `i`'s docids live
/// at posting index `i`). Stored ON DISK so a reopen drives range / exact /
/// boolean queries straight off the mmap WITHOUT rebuilding the in-RAM `values:
/// BTreeMap<SortableF64, RoaringBitmap>` index (which grows O(distinct numeric
/// values)).
const ROLE_NUMBER_SORTED: u8 = 13;
/// Number per-distinct-value INVERTED posting-block column (var-width,
/// LZ4-blocked). Phase 2h-3. Parallel to the [`ROLE_NUMBER_SORTED`] sorted-value
/// column: the entry at sorted index `i` is that value's sorted-docid posting
/// blob — the docids whose number EQUALS the `i`-th distinct value (a
/// delta-varint `docid`-gap stream with NO tf; see [`encode_docid_block`]). The
/// per-value doc-count (`df`) is the cheap LEB128 count prefix of the blob,
/// keeping the boolean planner's selectivity input off a full posting decode.
/// Located by the value's index in the sorted column (the binary-search result),
/// NOT by a string dict-id — Number has no [`ROLE_DICT`].
const ROLE_NUMBER_POSTINGS: u8 = 14;

/// Codec discriminant for [`ColumnRef::codec`]. A fixed-width column is stored
/// raw (zero-copy `try_cast_slice` on read); a var-width column is LZ4-blocked.
const CODEC_FIXED: u8 = 0;
/// LZ4-framed variable-width blocks (the var-column codec). Phase 2e-A.
const CODEC_LZ4_VAR: u8 = 1;

/// Sentinel dict-id meaning "no keyword for this doc" in the Keyword forward
/// column. `u32::MAX` can never be a real dict-id (a dict that large would
/// overflow the file long before reaching it), so it is an unambiguous absent
/// marker independent of the present bitset.
const DICT_ABSENT: u32 = u32::MAX;

/// Soft target for a var-column LZ4 block's *uncompressed* size: 64 KB. A block
/// is flushed once the accumulated raw bytes reach this cap; a single oversized
/// entry may exceed it.
const VAR_BLOCK_BYTES: usize = 64 * 1024;

/// Round `n` up to the next multiple of [`PAGE`].
#[inline]
fn page_align(n: usize) -> usize {
    (n + PAGE - 1) & !(PAGE - 1)
}

/// A directory entry locating one fixed-width column inside the file. CBOR is
/// only read once on open (cold path), so its overhead is irrelevant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnRef {
    pub name: String,
    /// `ROLE_*` discriminant identifying what the column holds.
    pub role: u8,
    /// `byte_offset` of the column's bytes. For a FIXED column this is the
    /// page-aligned start of the raw element array; for a VAR (LZ4) column it
    /// is the start of the column's first block in the VAR region.
    pub byte_offset: u64,
    pub byte_len: u64,
    /// FIXED: element count of the raw array. VAR: number of logical entries
    /// (dictionary cardinality) across all blocks.
    pub elem_count: u64,
    /// FIXED: element width in bytes. VAR: 0 (entries are variable width).
    pub width: u8,
    /// `CODEC_FIXED` (default — raw, zero-copy) or `CODEC_LZ4_VAR` (the
    /// var-width blocked codec). Defaults to `CODEC_FIXED` so a segment written
    /// before Phase 2e-A (no `codec` in its CBOR) decodes its fixed columns
    /// unchanged.
    #[serde(default)]
    pub codec: u8,
    /// VAR only: the per-column skip-index bytes (CBOR of [`SparseVarIndex`]),
    /// carried inline in the directory so a var read needs no second seek. A
    /// FIXED column leaves this empty.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub skip_index: Vec<u8>,
}

/// The fixed-size header fields (the on-disk header is this prefix followed by
/// zero padding out to [`HEADER_LEN`]). Not a bytemuck `Pod` because a 4072-byte
/// pad array exceeds bytemuck's derived-trait array support; the few scalar
/// fields are written/read explicitly little-endian via `byteorder` instead.
struct Header {
    magic1: u32,
    format_ver: u32,
    applied_seq: u64,
    host_endian_marker: u32,
    n_docs: u32,
    /// Phase 2e-B: BM25 corpus scalars for a Text segment (the `N` and the
    /// `total_doc_len` that derive `avgdl`). Zero for every non-text segment,
    /// and zero in any segment written before 2e-B — the header block is always
    /// the zero-padded [`HEADER_LEN`], so an older file reads these back as 0
    /// and stays decodable.
    doc_count: u64,
    total_doc_len: u64,
}

impl Header {
    /// Serialize the scalar fields (40 bytes) little-endian. The caller pads
    /// the remainder of the [`HEADER_LEN`] block with zeros. The first 24 bytes
    /// are byte-identical to the pre-2e-B layout; the two trailing `u64`s are
    /// appended so an older reader's 24-byte parse still works and a newer
    /// reader recovers the BM25 scalars (0 for a non-text segment).
    fn to_prefix_bytes(&self) -> [u8; 40] {
        let mut out = [0u8; 40];
        out[0..4].copy_from_slice(&self.magic1.to_le_bytes());
        out[4..8].copy_from_slice(&self.format_ver.to_le_bytes());
        out[8..16].copy_from_slice(&self.applied_seq.to_le_bytes());
        out[16..20].copy_from_slice(&self.host_endian_marker.to_le_bytes());
        out[20..24].copy_from_slice(&self.n_docs.to_le_bytes());
        out[24..32].copy_from_slice(&self.doc_count.to_le_bytes());
        out[32..40].copy_from_slice(&self.total_doc_len.to_le_bytes());
        out
    }

    /// Parse the scalar fields from the head of a header block. Requires the
    /// original 24-byte prefix; the two BM25 `u64`s default to 0 when the buffer
    /// is shorter (older segment, or torn tail of a header that is otherwise
    /// always [`HEADER_LEN`] zero-padded). Returns `Err` only when even the
    /// 24-byte prefix is missing (torn file) — never panics.
    fn from_bytes(buf: &[u8]) -> Result<Header> {
        if buf.len() < 24 {
            bail!("header too short: {} bytes", buf.len());
        }
        let mut cur = std::io::Cursor::new(buf);
        let magic1 = cur.read_u32::<LittleEndian>()?;
        let format_ver = cur.read_u32::<LittleEndian>()?;
        let applied_seq = cur.read_u64::<LittleEndian>()?;
        let host_endian_marker = cur.read_u32::<LittleEndian>()?;
        let n_docs = cur.read_u32::<LittleEndian>()?;
        // The two BM25 scalars live at bytes 24..40; absent (short buffer) => 0,
        // which is exactly a non-text / pre-2e-B segment's value.
        let doc_count = if buf.len() >= 32 {
            cur.read_u64::<LittleEndian>()?
        } else {
            0
        };
        let total_doc_len = if buf.len() >= 40 {
            cur.read_u64::<LittleEndian>()?
        } else {
            0
        };
        Ok(Header {
            magic1,
            format_ver,
            applied_seq,
            host_endian_marker,
            n_docs,
            doc_count,
            total_doc_len,
        })
    }
}

/// The fixed 24-byte footer at the file tail. Read field-by-field little-endian
/// rather than via a zero-copy cast: the footer starts at `file_len - 24`,
/// which is not 8-byte aligned for an arbitrary file length, so a `bytemuck`
/// cast to an alignment-8 struct would fail. A 24-byte cold read is cheap.
#[derive(Clone, Copy)]
struct Footer {
    dir_offset: u64,
    dir_len: u64,
    crc32: u32,
    magic2: u32,
}

impl Footer {
    /// Serialize the 24 footer bytes little-endian.
    fn to_bytes(&self) -> [u8; FOOTER_LEN] {
        let mut out = [0u8; FOOTER_LEN];
        out[0..8].copy_from_slice(&self.dir_offset.to_le_bytes());
        out[8..16].copy_from_slice(&self.dir_len.to_le_bytes());
        out[16..20].copy_from_slice(&self.crc32.to_le_bytes());
        out[20..24].copy_from_slice(&self.magic2.to_le_bytes());
        out
    }

    /// Parse the footer from a (>= 24-byte) tail slice. Returns `Err` on a
    /// short slice — never panics.
    fn from_bytes(buf: &[u8]) -> Result<Footer> {
        if buf.len() < FOOTER_LEN {
            bail!("footer too short: {} bytes", buf.len());
        }
        let mut cur = std::io::Cursor::new(buf);
        Ok(Footer {
            dir_offset: cur.read_u64::<LittleEndian>()?,
            dir_len: cur.read_u64::<LittleEndian>()?,
            crc32: cur.read_u32::<LittleEndian>()?,
            magic2: cur.read_u32::<LittleEndian>()?,
        })
    }
}

// ---------------------------------------------------------------------------
// Writer
// ---------------------------------------------------------------------------

/// Build the 4096-byte zero-padded header block for `n_docs` rows at
/// `applied_seq`. Non-text writers pass `doc_count = total_doc_len = 0` (the
/// BM25 scalars are only meaningful for a Text segment); a 2e-B Text segment
/// passes the corpus `N` and `Σ|d|` so the reader can derive `avgdl` from the
/// header alone. The first 24 header bytes are byte-identical regardless.
fn header_block(applied_seq: u64, n_docs: u32, doc_count: u64, total_doc_len: u64) -> Vec<u8> {
    let header = Header {
        magic1: MAGIC1,
        format_ver: FORMAT_VER,
        applied_seq,
        host_endian_marker: HOST_ENDIAN_MARKER,
        n_docs,
        doc_count,
        total_doc_len,
    };
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(&header.to_prefix_bytes());
    buf.resize(HEADER_LEN, 0); // zero-pad the rest of the header block
    debug_assert_eq!(buf.len(), HEADER_LEN);
    buf
}

/// Append a `u64[ceil(n/64)]` present bitset (bit `i` set == doc `i` present)
/// to `buf`, returning `(byte_offset, byte_len, n_words)`. Shared by every
/// fixed-width writer.
fn append_present_bitset(buf: &mut Vec<u8>, present: &[bool]) -> Result<(u64, u64, u64)> {
    // The present bitset is a `u64` column, so its start must be 8-aligned for a
    // zero-copy `try_cast_slice::<u8, u64>` on read. The u64 Number/Hash forward
    // columns already end 8-aligned (this pad is a no-op there, so their bytes
    // are unchanged); the f32 vector column can end mid-word and needs it.
    let pad = (8 - (buf.len() % 8)) % 8;
    buf.resize(buf.len() + pad, 0);
    let off = buf.len();
    let n_words = (present.len() + 63) / 64;
    let mut words = vec![0u64; n_words];
    for (i, &p) in present.iter().enumerate() {
        if p {
            words[i / 64] |= 1u64 << (i % 64);
        }
    }
    let bytes: &[u8] = bytemuck::try_cast_slice(&words)
        .map_err(|e| anyhow!("cast present bitset to bytes: {e:?}"))?;
    buf.extend_from_slice(bytes);
    let len = words.len() * std::mem::size_of::<u64>();
    Ok((off as u64, len as u64, n_words as u64))
}

/// Build the directory entry for a present bitset. Authored once so every
/// writer emits an identical `present` [`ColumnRef`] (fixed-width, u64 words).
fn present_column_ref(off: u64, len: u64, n_words: u64) -> ColumnRef {
    ColumnRef {
        name: "present".to_string(),
        role: ROLE_PRESENT,
        byte_offset: off,
        byte_len: len,
        elem_count: n_words,
        width: 8,
        codec: CODEC_FIXED,
        skip_index: Vec::new(),
    }
}

/// Finalize an assembled file: page-pad the fixed-width region tail, append the
/// CBOR directory (crc'd), append the 24-byte footer, then atomically write
/// `path` via a temp file + rename (mirrors `rdb.rs`). Shared by every writer
/// so the header/dir/footer/atomic-write tail is authored exactly once.
fn finalize_and_write(path: &Path, mut buf: Vec<u8>, dir: Vec<ColumnRef>) -> Result<()> {
    // Pad the fixed-width region tail to the next page boundary.
    let region_end = page_align(buf.len());
    buf.resize(region_end, 0);

    // --- DIRECTORY (CBOR) ---
    let mut dir_bytes: Vec<u8> = Vec::new();
    ciborium::into_writer(&dir, &mut dir_bytes)
        .map_err(|e| anyhow!("cbor encode segment directory: {e}"))?;

    let dir_offset = buf.len() as u64;
    let dir_len = dir_bytes.len() as u64;
    let dir_crc = crc32fast::hash(&dir_bytes);
    buf.extend_from_slice(&dir_bytes);

    // --- FOOTER (24 bytes, very tail) ---
    let footer = Footer {
        dir_offset,
        dir_len,
        crc32: dir_crc,
        magic2: MAGIC2,
    };
    buf.extend_from_slice(&footer.to_bytes());

    // --- Atomic write: temp file + rename (mirror rdb.rs). ---
    let dir_path = path
        .parent()
        .ok_or_else(|| anyhow!("segment path has no parent: {}", path.display()))?;
    std::fs::create_dir_all(dir_path)
        .with_context(|| format!("create segment dir {}", dir_path.display()))?;
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| anyhow!("segment path has no file name: {}", path.display()))?;
    let tmp = dir_path.join(format!(".{file_name}.tmp"));

    {
        let mut f = File::create(&tmp).with_context(|| format!("create {}", tmp.display()))?;
        f.write_all(&buf)
            .with_context(|| format!("write {}", tmp.display()))?;
        f.sync_all()
            .with_context(|| format!("fsync {}", tmp.display()))?;
    }
    std::fs::rename(&tmp, path)
        .with_context(|| format!("rename {} -> {}", tmp.display(), path.display()))?;
    Ok(())
}

/// The order-preserving `SortableF64` bit key of a raw `f64` (Phase 2h-3).
/// MUST byte-match `storage::SortableF64::new(x)`'s inner `u64`: a non-negative
/// value flips the top bit (placing it above negatives); a negative flips all
/// bits (reversing magnitude order among negatives). The resulting `u64` is
/// MONOTONE in numeric order, so an ascending `u64` column is an ascending
/// numeric column. NaN can never reach the seal (rejected at index time), so we
/// do not special-case it here — the seal only feeds finite values.
#[inline]
fn sortable_bits(x: f64) -> u64 {
    let bits = x.to_bits();
    if x.is_sign_negative() {
        !bits
    } else {
        bits ^ (1u64 << 63)
    }
}

/// Write a Number column for `values` (one entry per doc id, `None` = absent)
/// to `path`, tagged with `applied_seq`. Emits the segment format:
/// 4096-byte header, page-aligned `u64[n_docs]` forward column, present
/// bitset, then (Phase 2h-3) a SORTED-VALUE range index — a fixed-width
/// ascending `u64[distinct]` column of the distinct `SortableF64` bit keys
/// ([`ROLE_NUMBER_SORTED`]) plus a parallel per-distinct-value docid posting
/// column ([`ROLE_NUMBER_POSTINGS`]) — page-padded region tail, VAR region,
/// CBOR directory, 24-byte footer. Writes to a temp file then atomically
/// renames into place (mirrors `rdb.rs`).
///
/// Phase 2h-3: the in-RAM inverted/range `values: BTreeMap<SortableF64,
/// RoaringBitmap>` now lives ON DISK. The sorted-value column + postings are
/// DERIVED from `values` here (folded over the same per-doc forward state the
/// in-RAM `values` map was built from), so a reopen drives range / exact /
/// boolean queries straight off the mmap WITHOUT rebuilding `values` in RAM.
/// `values[id]` is doc `id`'s live value (a deleted base doc is `None`, so it is
/// excluded from BOTH the forward column AND the sorted index), making the
/// on-disk index exactly the live in-RAM `values` snapshot.
pub fn write_number_segment(path: &Path, applied_seq: u64, values: &[Option<f64>]) -> Result<()> {
    let n_docs: u32 = values
        .len()
        .try_into()
        .context("segment exceeds u32 doc capacity")?;

    // Fold the forward column into the SORTED distinct-value range index: each
    // distinct `SortableF64` bit key -> its ascending docid list. BTreeMap keyed
    // on the raw `u64` bit key gives ascending numeric order (the key is monotone
    // in value), exactly reproducing the in-RAM `values` BTreeMap's key order and
    // per-key ascending-docid posting (RoaringBitmap iterates sorted).
    let mut sorted: std::collections::BTreeMap<u64, Vec<u32>> = std::collections::BTreeMap::new();
    for (id, v) in values.iter().enumerate() {
        if let Some(x) = v {
            sorted.entry(sortable_bits(*x)).or_default().push(id as u32);
        }
    }
    // Distinct keys ascending; parallel per-value docid-only posting blobs.
    let sorted_keys: Vec<u64> = sorted.keys().copied().collect();
    let posting_blobs: Vec<Vec<u8>> = sorted
        .values()
        .map(|docids| encode_docid_block(docids))
        .collect();

    let mut buf = header_block(applied_seq, n_docs, 0, 0);

    // --- FIXED-WIDTH REGION ---
    // Pad up to the page-aligned start of the Number forward column.
    let number_off = page_align(buf.len());
    buf.resize(number_off, 0);

    // Number forward column: u64[n_docs] = f64::to_bits of each value (0 for absent).
    let number_words: Vec<u64> = values
        .iter()
        .map(|v| v.map(f64::to_bits).unwrap_or(0))
        .collect();
    // try_cast_slice (never cast_slice): u64 -> u8 always succeeds, but we
    // honor the no-panic discipline everywhere.
    let number_col_bytes: &[u8] = bytemuck::try_cast_slice(&number_words)
        .map_err(|e| anyhow!("cast number column to bytes: {e:?}"))?;
    buf.extend_from_slice(number_col_bytes);
    let number_len = number_words.len() * std::mem::size_of::<u64>();

    // SORTED-VALUE range index column (Phase 2h-3): u64[distinct] ascending bit
    // keys. The forward column ends 8-aligned, so this u64 column is already
    // 8-aligned for a zero-copy `try_cast_slice::<u8, u64>` on read.
    let sorted_col_bytes: &[u8] = bytemuck::try_cast_slice(&sorted_keys)
        .map_err(|e| anyhow!("cast sorted-value column to bytes: {e:?}"))?;
    let sorted_off = buf.len() as u64;
    buf.extend_from_slice(sorted_col_bytes);
    let sorted_len = (sorted_keys.len() * std::mem::size_of::<u64>()) as u64;

    // Present bitset: u64[ceil(n_docs/64)], bit i set == doc i has a value.
    let present: Vec<bool> = values.iter().map(|v| v.is_some()).collect();
    let (present_off, present_len, n_words) = append_present_bitset(&mut buf, &present)?;

    // --- VAR REGION (after the page-padded fixed region) ---
    let region_end = page_align(buf.len());
    buf.resize(region_end, 0);
    let postings_ref = append_var_blob_column(
        &mut buf,
        &posting_blobs,
        ROLE_NUMBER_POSTINGS,
        "number_postings",
    )?;

    let dir = vec![
        ColumnRef {
            name: "number".to_string(),
            role: ROLE_NUMBER,
            byte_offset: number_off as u64,
            byte_len: number_len as u64,
            elem_count: n_docs as u64,
            width: 8,
            codec: CODEC_FIXED,
            skip_index: Vec::new(),
        },
        ColumnRef {
            name: "number_sorted".to_string(),
            role: ROLE_NUMBER_SORTED,
            byte_offset: sorted_off,
            byte_len: sorted_len,
            elem_count: sorted_keys.len() as u64,
            width: 8,
            codec: CODEC_FIXED,
            skip_index: Vec::new(),
        },
        present_column_ref(present_off, present_len, n_words),
        postings_ref,
    ];
    finalize_and_write(path, buf, dir)
}

/// Write a Hash column for `values` (one `u64` perceptual hash per doc id,
/// `None` = absent) to `path`. Same layout as the Number column but the
/// forward column stores the raw `u64` hash directly (no f64-bits transform) —
/// role [`ROLE_HASH`]. Mirrors [`write_number_segment`] for everything else.
pub fn write_hash_segment(path: &Path, applied_seq: u64, values: &[Option<u64>]) -> Result<()> {
    let n_docs: u32 = values
        .len()
        .try_into()
        .context("segment exceeds u32 doc capacity")?;

    let mut buf = header_block(applied_seq, n_docs, 0, 0);

    // --- FIXED-WIDTH REGION ---
    let hash_off = page_align(buf.len());
    buf.resize(hash_off, 0);

    // Hash forward column: u64[n_docs] = the raw hash (0 for absent).
    let hash_words: Vec<u64> = values.iter().map(|v| v.unwrap_or(0)).collect();
    let hash_col_bytes: &[u8] = bytemuck::try_cast_slice(&hash_words)
        .map_err(|e| anyhow!("cast hash column to bytes: {e:?}"))?;
    buf.extend_from_slice(hash_col_bytes);
    let hash_len = hash_words.len() * std::mem::size_of::<u64>();

    let present: Vec<bool> = values.iter().map(|v| v.is_some()).collect();
    let (present_off, present_len, n_words) = append_present_bitset(&mut buf, &present)?;

    let dir = vec![
        ColumnRef {
            name: "hash".to_string(),
            role: ROLE_HASH,
            byte_offset: hash_off as u64,
            byte_len: hash_len as u64,
            elem_count: n_docs as u64,
            width: 8,
            codec: CODEC_FIXED,
            skip_index: Vec::new(),
        },
        present_column_ref(present_off, present_len, n_words),
    ];
    finalize_and_write(path, buf, dir)
}

/// Write a Vector column: a contiguous, decoded `f32[n_docs * dim]` forward
/// column plus a present bitset. `vectors[i]` is doc `i`'s vector (`None` =
/// absent, written as `dim` zeros and bit clear in the present set). Every
/// present slice MUST be exactly `dim` long. Role [`ROLE_VECTOR`], width 4. No
/// scalar quantization — the bytes on disk are the exact `f32` bits, so a
/// zero-copy scan is bit-identical to the in-RAM corpus.
pub fn write_vector_segment(
    path: &Path,
    applied_seq: u64,
    dim: usize,
    vectors: &[Option<&[f32]>],
) -> Result<()> {
    let n_docs: u32 = vectors
        .len()
        .try_into()
        .context("segment exceeds u32 doc capacity")?;
    if dim == 0 {
        bail!("vector segment dim must be > 0");
    }

    let mut buf = header_block(applied_seq, n_docs, 0, 0);

    // --- FIXED-WIDTH REGION ---
    let vector_off = page_align(buf.len());
    buf.resize(vector_off, 0);

    // Vector forward column: f32[n_docs * dim], dense in docid order. An absent
    // doc contributes `dim` zeros (and a clear present bit).
    let zeros = vec![0f32; dim];
    let mut data: Vec<f32> = Vec::with_capacity(vectors.len() * dim);
    for v in vectors {
        match v {
            Some(slice) => {
                if slice.len() != dim {
                    bail!("vector has dim {} but segment dim is {dim}", slice.len());
                }
                data.extend_from_slice(slice);
            }
            None => data.extend_from_slice(&zeros),
        }
    }
    let vector_col_bytes: &[u8] = bytemuck::try_cast_slice(&data)
        .map_err(|e| anyhow!("cast vector column to bytes: {e:?}"))?;
    buf.extend_from_slice(vector_col_bytes);
    let vector_len = data.len() * std::mem::size_of::<f32>();

    let present: Vec<bool> = vectors.iter().map(|v| v.is_some()).collect();
    let (present_off, present_len, n_words) = append_present_bitset(&mut buf, &present)?;

    let dir = vec![
        ColumnRef {
            name: "vector".to_string(),
            role: ROLE_VECTOR,
            byte_offset: vector_off as u64,
            byte_len: vector_len as u64,
            elem_count: (n_docs as u64) * (dim as u64),
            width: 4,
            codec: CODEC_FIXED,
            skip_index: Vec::new(),
        },
        present_column_ref(present_off, present_len, n_words),
    ];
    finalize_and_write(path, buf, dir)
}

// ---------------------------------------------------------------------------
// Variable-width column machinery (Phase 2e-A)
// ---------------------------------------------------------------------------
//
// A var-width column is a sorted run of byte-string entries (the Keyword / Set
// string dictionaries) stored as a sequence of LZ4-framed blocks in the VAR
// region (after the page-padded fixed region). Each block holds up to
// `VAR_BLOCK_BYTES` of raw, prefix-delta-encoded entries; the per-column
// skip-index ([`SparseVarIndex`]) carries one [`VarBlockMeta`] per block so a
// lookup binary-searches to the owning block, decompresses it once (caching it
// in the moka byte-weighted cache on the reader), then scans the block.
//
// The framing: LZ4 64KB blocks (`u32` len + `lz4_flex::compress_prepend_size`
// / `decompress_size_prepended`), a sparse skip-index (`BlockMeta` +
// binary-search `locate`), and a prefix-delta string encoder
// (`delta_encode` / `shared_prefix`). No WAL/fsync/per-record-crc and no
// memtable — a segment is NATS-durable and discard-on-torn, so the only
// integrity gate is the directory footer crc32.

/// Per-column skip-index over a var-width column's LZ4 blocks. CBOR-encoded
/// into [`ColumnRef::skip_index`] so the reader never seeks for it.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct SparseVarIndex {
    blocks: Vec<VarBlockMeta>,
}

/// Locates one LZ4 block of a var-width column and records the logical entry
/// id range it covers. `first_entry` is the dict-id of the block's first entry
/// (entries are globally id-ordered, so a target id binary-searches to the
/// block whose `first_entry <= id`).
#[derive(Debug, Clone, Serialize, Deserialize)]
struct VarBlockMeta {
    /// Dict-id of the first entry stored in this block.
    first_entry: u32,
    /// Number of entries packed into this block.
    entry_count: u32,
    /// Byte offset of the block's `u32` compressed-length prefix, relative to
    /// the file start.
    offset: u64,
    /// Compressed length on disk (the bytes after the `u32` length prefix).
    length: u32,
}

/// The shared-prefix length of two byte strings (the delta-encode leg).
fn shared_prefix(a: &[u8], b: &[u8]) -> usize {
    a.iter().zip(b.iter()).take_while(|(x, y)| x == y).count()
}

/// One prefix-delta entry inside a decoded var block.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct VarEntry {
    /// Bytes shared with the previous entry in the block (delta-encode).
    shared: u32,
    /// New bytes appended after the shared prefix.
    suffix: Vec<u8>,
}

/// The decoded body of one var block: prefix-delta entries. A whole block is
/// CBOR-encoded then LZ4-compressed; on read it is decompressed once and the
/// entries are reconstructed by rolling the shared prefix forward.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct VarBlockBody {
    entries: Vec<VarEntry>,
}

impl VarBlockBody {
    /// Reconstruct the full byte strings in this block (rolling the shared
    /// prefix forward). Returns one `Vec<u8>` per entry, in id order.
    fn reconstruct(&self) -> Vec<Vec<u8>> {
        let mut out = Vec::with_capacity(self.entries.len());
        let mut prev: Vec<u8> = Vec::new();
        for e in &self.entries {
            let shared = (e.shared as usize).min(prev.len());
            let mut s = prev[..shared].to_vec();
            s.extend_from_slice(&e.suffix);
            out.push(s.clone());
            prev = s;
        }
        out
    }
}

/// Accumulates a var-width column into LZ4 64KB blocks appended to `buf`,
/// building the per-column skip-index as it goes. Entries are pushed in
/// dict-id order (`0, 1, 2, ...`); the writer flushes a block once the
/// accumulated *raw* bytes reach [`VAR_BLOCK_BYTES`].
struct VarColumnWriter {
    /// Prefix-delta entries pending in the current (not-yet-flushed) block.
    pending: Vec<VarEntry>,
    /// Raw byte count accumulated in `pending` (drives the 64KB flush cap).
    pending_bytes: usize,
    /// Previous entry's full bytes, for the prefix delta. Reset per block so a
    /// block decodes self-contained.
    prev: Vec<u8>,
    /// Dict-id of the first entry in the current block.
    block_first: u32,
    /// Total entries pushed so far (== dict-id of the next entry).
    next_id: u32,
    /// Skip-index accumulated as blocks flush.
    index: Vec<VarBlockMeta>,
}

impl VarColumnWriter {
    fn new() -> Self {
        Self {
            pending: Vec::new(),
            pending_bytes: 0,
            prev: Vec::new(),
            block_first: 0,
            next_id: 0,
            index: Vec::new(),
        }
    }

    /// Push the next dictionary entry (in id order). Prefix-deltas it against
    /// the previous entry IN THE SAME BLOCK, then flushes the block to `buf`
    /// when the raw byte budget is reached.
    fn push(&mut self, entry: &[u8], buf: &mut Vec<u8>) -> Result<()> {
        let shared = shared_prefix(&self.prev, entry) as u32;
        let suffix = entry[shared as usize..].to_vec();
        self.pending_bytes += 4 + suffix.len();
        self.pending.push(VarEntry { shared, suffix });
        self.prev = entry.to_vec();
        self.next_id += 1;
        if self.pending_bytes >= VAR_BLOCK_BYTES {
            self.flush_block(buf)?;
        }
        Ok(())
    }

    /// Flush the pending entries as one LZ4-framed block: `u32` compressed
    /// length + `lz4_flex::compress_prepend_size(cbor(body))`. Records a
    /// [`VarBlockMeta`] in the skip-index. No-op when empty.
    fn flush_block(&mut self, buf: &mut Vec<u8>) -> Result<()> {
        if self.pending.is_empty() {
            return Ok(());
        }
        let body = VarBlockBody {
            entries: std::mem::take(&mut self.pending),
        };
        let mut raw = Vec::new();
        ciborium::into_writer(&body, &mut raw).map_err(|e| anyhow!("encode var block: {e}"))?;
        let compressed = lz4_flex::compress_prepend_size(&raw);
        let offset = buf.len() as u64;
        buf.extend_from_slice(&(compressed.len() as u32).to_le_bytes());
        buf.extend_from_slice(&compressed);
        self.index.push(VarBlockMeta {
            first_entry: self.block_first,
            entry_count: body.entries.len() as u32,
            offset,
            length: compressed.len() as u32,
        });
        // Reset for the next block: blocks decode self-contained, so the prefix
        // delta restarts from empty.
        self.pending_bytes = 0;
        self.prev = Vec::new();
        self.block_first = self.next_id;
        Ok(())
    }

    /// Flush any trailing partial block and return the encoded skip-index +
    /// the column's `(byte_offset, byte_len)` span. `byte_offset` is the start
    /// of the column's first block (captured before the first `push`).
    fn finish(mut self, buf: &mut Vec<u8>, byte_offset: u64) -> Result<(Vec<u8>, u64, u64)> {
        self.flush_block(buf)?;
        let index = SparseVarIndex { blocks: self.index };
        let mut idx_bytes = Vec::new();
        ciborium::into_writer(&index, &mut idx_bytes)
            .map_err(|e| anyhow!("encode var skip-index: {e}"))?;
        let byte_len = buf.len() as u64 - byte_offset;
        Ok((idx_bytes, byte_offset, byte_len))
    }
}

/// Write a sorted byte-string dictionary `dict` (entry `i` has dict-id `i`) as
/// a var-width LZ4 column appended to `buf`, returning its [`ColumnRef`]
/// (role [`ROLE_DICT`], codec [`CODEC_LZ4_VAR`], skip-index carried inline).
fn append_dict_column(buf: &mut Vec<u8>, dict: &[Vec<u8>]) -> Result<ColumnRef> {
    let byte_offset = buf.len() as u64;
    let mut w = VarColumnWriter::new();
    for entry in dict {
        w.push(entry, buf)?;
    }
    let (skip_index, off, byte_len) = w.finish(buf, byte_offset)?;
    Ok(ColumnRef {
        name: "dict".to_string(),
        role: ROLE_DICT,
        byte_offset: off,
        byte_len,
        elem_count: dict.len() as u64,
        width: 0,
        codec: CODEC_LZ4_VAR,
        skip_index,
    })
}

// ---------------------------------------------------------------------------
// Text posting-block codec (Phase 2e-B)
// ---------------------------------------------------------------------------
//
// Text term-frequency is NOT rebuildable from a forward column the way the
// Keyword/Set inverted indexes are, so one token's posting list is STORED as a
// self-describing blob — the "entry" pushed into the shared [`VarColumnWriter`]
// at the token's dict-id. The blob is a tight LEB128 delta-varint stream:
//
//   [count : LEB128]
//   then `count` postings, in the SAME ascending-docid order they hold in the
//   live `Postings` SoA, each as:
//     [docid_gap : LEB128]   (gap from the previous docid; first gap == docid0)
//     [tf        : LEB128]
//
// Ascending docids make the gaps small; LZ4 then squeezes the blob in the var
// block. `decode_posting_block` rebuilds `(docids, tfs)` by rolling the gap
// forward, reproducing the exact streams the live BM25 scan reads.

/// Append `v` to `out` as an unsigned LEB128 varint.
fn write_varint(out: &mut Vec<u8>, mut v: u64) {
    loop {
        let byte = (v & 0x7f) as u8;
        v >>= 7;
        if v == 0 {
            out.push(byte);
            break;
        }
        out.push(byte | 0x80);
    }
}

/// Read an unsigned LEB128 varint from `buf` at `*pos`, advancing `*pos`.
/// `None` on a truncated / overlong (>10-byte) varint — never panics.
fn read_varint(buf: &[u8], pos: &mut usize) -> Option<u64> {
    let mut result: u64 = 0;
    let mut shift: u32 = 0;
    loop {
        let byte = *buf.get(*pos)?;
        *pos += 1;
        // A u64 is at most 10 LEB128 bytes; reject an overlong run rather than
        // silently wrapping the shift.
        if shift >= 64 {
            return None;
        }
        result |= ((byte & 0x7f) as u64) << shift;
        if byte & 0x80 == 0 {
            return Some(result);
        }
        shift += 7;
    }
}

/// Encode one token's `(docids, tfs)` postings into a delta-varint blob. The
/// caller guarantees `docids` is ascending and `docids.len() == tfs.len()`.
fn encode_posting_block(docids: &[u32], tfs: &[u32]) -> Vec<u8> {
    debug_assert_eq!(docids.len(), tfs.len());
    let mut out = Vec::with_capacity(docids.len() * 2 + 2);
    write_varint(&mut out, docids.len() as u64);
    let mut prev: u32 = 0;
    for (i, &id) in docids.iter().enumerate() {
        // First gap is `id - 0 == id`; subsequent gaps are strictly positive
        // (docids ascending & distinct).
        let gap = id.wrapping_sub(prev);
        write_varint(&mut out, gap as u64);
        write_varint(&mut out, tfs[i] as u64);
        prev = id;
    }
    out
}

/// Decode a posting blob back into `(docids, tfs)`. `None` on a truncated /
/// malformed blob — never panics. Reproduces the exact ascending-docid streams
/// [`encode_posting_block`] consumed.
fn decode_posting_block(blob: &[u8]) -> Option<(Vec<u32>, Vec<u32>)> {
    let mut pos = 0usize;
    let count = read_varint(blob, &mut pos)? as usize;
    let mut docids = Vec::with_capacity(count);
    let mut tfs = Vec::with_capacity(count);
    let mut prev: u32 = 0;
    for _ in 0..count {
        let gap = u32::try_from(read_varint(blob, &mut pos)?).ok()?;
        let tf = u32::try_from(read_varint(blob, &mut pos)?).ok()?;
        let id = prev.checked_add(gap)?;
        docids.push(id);
        tfs.push(tf);
        prev = id;
    }
    Some((docids, tfs))
}

// ---------------------------------------------------------------------------
// Keyword docid-only posting-block codec (Phase 2h-1)
// ---------------------------------------------------------------------------
//
// A Keyword (or Set) term holds a doc at most once, so — unlike Text — there is
// NO term-frequency to carry. A term's posting list is therefore stored as a
// docid-only delta-varint blob (the "entry" pushed into the shared
// [`VarColumnWriter`] at the term's dict-id):
//
//   [count : LEB128]
//   then `count` ascending docids, each as:
//     [docid_gap : LEB128]   (gap from the previous docid; first gap == docid0)
//
// `count` is the term's document frequency (`df`), readable on its own — the
// boolean planner's rarest-first ordering only needs `df`, so it decodes just
// this prefix (mirrors `text_token_df`). `decode_docid_block` rebuilds the
// ascending docid stream, which is fed straight into a `RoaringBitmap` so the
// segment-driven AND/OR algebra is bit-identical to the in-RAM `terms` bitmap.

/// Encode a term's ascending `docids` into a count-prefixed delta-varint blob.
/// The caller guarantees `docids` is strictly ascending and distinct. Mirrors
/// [`encode_posting_block`] minus the per-doc `tf` leg.
fn encode_docid_block(docids: &[u32]) -> Vec<u8> {
    let mut out = Vec::with_capacity(docids.len() + 2);
    write_varint(&mut out, docids.len() as u64);
    let mut prev: u32 = 0;
    for &id in docids {
        // First gap is `id - 0 == id`; subsequent gaps are strictly positive
        // (docids ascending & distinct).
        let gap = id.wrapping_sub(prev);
        write_varint(&mut out, gap as u64);
        prev = id;
    }
    out
}

/// Decode a docid-only posting blob back into the ascending docid stream. `None`
/// on a truncated / malformed blob — never panics. Reproduces the exact
/// ascending-docid stream [`encode_docid_block`] consumed.
fn decode_docid_block(blob: &[u8]) -> Option<Vec<u32>> {
    let mut pos = 0usize;
    let count = read_varint(blob, &mut pos)? as usize;
    let mut docids = Vec::with_capacity(count);
    let mut prev: u32 = 0;
    for _ in 0..count {
        let gap = u32::try_from(read_varint(blob, &mut pos)?).ok()?;
        let id = prev.checked_add(gap)?;
        docids.push(id);
        prev = id;
    }
    Some(docids)
}

/// Append a fixed-width `u32` column (one of the Keyword/Set id/offset/packed
/// columns) to `buf`, returning `(byte_offset, byte_len)`. The column start is
/// 4-aligned so `try_cast_slice::<u8, u32>` succeeds on read; the page-aligned
/// fixed-region start plus 4-aligned running padding keeps every `u32` column
/// 4-aligned.
fn append_u32_column(buf: &mut Vec<u8>, values: &[u32]) -> Result<(u64, u64)> {
    let pad = (4 - (buf.len() % 4)) % 4;
    buf.resize(buf.len() + pad, 0);
    let off = buf.len() as u64;
    let bytes: &[u8] =
        bytemuck::try_cast_slice(values).map_err(|e| anyhow!("cast u32 column: {e:?}"))?;
    buf.extend_from_slice(bytes);
    let len = (values.len() * std::mem::size_of::<u32>()) as u64;
    Ok((off, len))
}

/// Write a Keyword segment: a sorted prefix-compressed string DICT (var-width)
/// + a FIXED `u32[n_docs]` dict-id forward column ([`DICT_ABSENT`] for a doc
/// with no keyword) + a parallel per-term INVERTED posting-block column
/// ([`ROLE_KEYWORD_POSTINGS`]) + a present bitset. `values[i]` is doc `i`'s
/// keyword (`None` = absent).
///
/// Phase 2h-1: the inverted `terms` index now lives ON DISK — for each dict
/// term (in dict-id / sorted order) a docid-only delta-varint posting blob is
/// pushed into a var column parallel to the dictionary. `postings` is the live
/// `terms: BTreeMap<String, RoaringBitmap>`; its keys MUST equal `values`'s
/// distinct non-`None` strings (the seal builds both from the same forward
/// state). The blobs are written in DICT order (BTreeSet ascending), which is
/// exactly `postings`'s BTreeMap key order, so term `t`'s dict index locates
/// its posting block. On reopen the inverted index is NOT rebuilt in RAM — the
/// reader serves Term/Terms/df straight off this column.
pub fn write_keyword_segment(
    path: &Path,
    applied_seq: u64,
    values: &[Option<&str>],
    postings: &std::collections::BTreeMap<String, roaring::RoaringBitmap>,
) -> Result<()> {
    let n_docs: u32 = values
        .len()
        .try_into()
        .context("segment exceeds u32 doc capacity")?;

    // Build the sorted distinct dictionary; map each distinct string -> dict-id.
    let mut distinct: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();
    for v in values {
        if let Some(s) = v {
            distinct.insert(s);
        }
    }
    let dict: Vec<Vec<u8>> = distinct.iter().map(|s| s.as_bytes().to_vec()).collect();
    let dict_id: std::collections::HashMap<&str, u32> = distinct
        .iter()
        .enumerate()
        .map(|(i, s)| (*s, i as u32))
        .collect();

    // Per-term posting blobs in DICT order. `distinct` is the sorted set of the
    // forward values; `postings` is keyed by the same strings (both derive from
    // the same live forward state at seal), so `postings[term]` always exists.
    // Each term's bitmap is ascending-docid by construction (RoaringBitmap
    // iterates sorted), matching `encode_docid_block`'s contract.
    let blobs: Vec<Vec<u8>> = distinct
        .iter()
        .map(|term| {
            let docids: Vec<u32> = postings
                .get(*term)
                .map(|bm| bm.iter().collect())
                .unwrap_or_default();
            encode_docid_block(&docids)
        })
        .collect();

    let mut buf = header_block(applied_seq, n_docs, 0, 0);

    // --- FIXED REGION ---
    // Forward dict-id column: u32[n_docs], DICT_ABSENT for an absent doc.
    let dictid_start = page_align(buf.len());
    buf.resize(dictid_start, 0);
    let ids: Vec<u32> = values
        .iter()
        .map(|v| v.map(|s| dict_id[s]).unwrap_or(DICT_ABSENT))
        .collect();
    let (dictid_off, dictid_len) = append_u32_column(&mut buf, &ids)?;

    // Present bitset.
    let present: Vec<bool> = values.iter().map(|v| v.is_some()).collect();
    let (present_off, present_len, n_words) = append_present_bitset(&mut buf, &present)?;

    // --- VAR REGION (after the page-padded fixed region) ---
    let region_end = page_align(buf.len());
    buf.resize(region_end, 0);
    let dict_ref = append_dict_column(&mut buf, &dict)?;
    let postings_ref =
        append_var_blob_column(&mut buf, &blobs, ROLE_KEYWORD_POSTINGS, "keyword_postings")?;

    let dir = vec![
        ColumnRef {
            name: "keyword_dictid".to_string(),
            role: ROLE_KEYWORD_DICTID,
            byte_offset: dictid_off,
            byte_len: dictid_len,
            elem_count: n_docs as u64,
            width: 4,
            codec: CODEC_FIXED,
            skip_index: Vec::new(),
        },
        present_column_ref(present_off, present_len, n_words),
        dict_ref,
        postings_ref,
    ];
    finalize_and_write(path, buf, dir)
}

/// Write a Set segment: a shared sorted string DICT (var-width) + a FIXED
/// `u32[n_docs + 1]` CSR offsets column + a FIXED `u32[total_members]` packed
/// dict-id column + a parallel per-element INVERTED posting-block column
/// ([`ROLE_SET_POSTINGS`]) + a present bitset. Doc `i`'s members are
/// `packed[offsets[i]..offsets[i+1]]` (CSR). `values[i]` is doc `i`'s members
/// in ascending order (`None` = the doc has no set value at all; an empty
/// `Some(&[])` is a present-but-empty set).
///
/// Phase 2h-2: the inverted `elements` index now lives ON DISK — for each dict
/// element (in dict-id / sorted order) a docid-only delta-varint posting blob
/// (the docids whose set contains that element) is pushed into a var column
/// parallel to the dictionary. `postings` is the live
/// `elements: BTreeMap<String, RoaringBitmap>`; its keys MUST equal `values`'s
/// distinct members (the seal builds both from the same forward state). The
/// blobs are written in DICT order (BTreeSet ascending), which is exactly
/// `postings`'s BTreeMap key order, so element `t`'s dict index locates its
/// posting block. On reopen the inverted index is NOT rebuilt in RAM — the
/// reader serves membership / Terms / df straight off this column.
pub fn write_set_segment(
    path: &Path,
    applied_seq: u64,
    values: &[Option<&[String]>],
    postings: &std::collections::BTreeMap<String, roaring::RoaringBitmap>,
) -> Result<()> {
    let n_docs: u32 = values
        .len()
        .try_into()
        .context("segment exceeds u32 doc capacity")?;

    // Shared distinct dictionary across all members of all docs.
    let mut distinct: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();
    for v in values.iter().flatten() {
        for m in v.iter() {
            distinct.insert(m.as_str());
        }
    }
    let dict: Vec<Vec<u8>> = distinct.iter().map(|s| s.as_bytes().to_vec()).collect();
    let dict_id: std::collections::HashMap<&str, u32> = distinct
        .iter()
        .enumerate()
        .map(|(i, s)| (*s, i as u32))
        .collect();

    // CSR offsets[n_docs + 1] + packed dict-ids. offsets[0] = 0; offsets[i+1] =
    // offsets[i] + |doc i's members|. An absent doc contributes 0 members (its
    // slice offsets[i]..offsets[i+1] is empty) and is also clear in `present`.
    let mut offsets: Vec<u32> = Vec::with_capacity(values.len() + 1);
    let mut packed: Vec<u32> = Vec::new();
    offsets.push(0);
    for v in values {
        if let Some(members) = v {
            for m in members.iter() {
                packed.push(dict_id[m.as_str()]);
            }
        }
        offsets.push(packed.len() as u32);
    }

    // Per-element INVERTED posting blobs in DICT order (Phase 2h-2). `distinct`
    // is the sorted set of all members; `postings` is keyed by the same strings
    // (both derive from the same live forward state at seal), so
    // `postings[element]` always exists. Each element's bitmap is ascending-docid
    // by construction (RoaringBitmap iterates sorted), matching
    // `encode_docid_block`'s contract.
    let blobs: Vec<Vec<u8>> = distinct
        .iter()
        .map(|el| {
            let docids: Vec<u32> = postings
                .get(*el)
                .map(|bm| bm.iter().collect())
                .unwrap_or_default();
            encode_docid_block(&docids)
        })
        .collect();

    let mut buf = header_block(applied_seq, n_docs, 0, 0);

    // --- FIXED REGION ---
    let offsets_start = page_align(buf.len());
    buf.resize(offsets_start, 0);
    let (offsets_off, offsets_len) = append_u32_column(&mut buf, &offsets)?;
    let (packed_off, packed_len) = append_u32_column(&mut buf, &packed)?;

    let present: Vec<bool> = values.iter().map(|v| v.is_some()).collect();
    let (present_off, present_len, n_words) = append_present_bitset(&mut buf, &present)?;

    // --- VAR REGION ---
    let region_end = page_align(buf.len());
    buf.resize(region_end, 0);
    let dict_ref = append_dict_column(&mut buf, &dict)?;
    let postings_ref = append_var_blob_column(&mut buf, &blobs, ROLE_SET_POSTINGS, "set_postings")?;

    let dir = vec![
        ColumnRef {
            name: "set_offsets".to_string(),
            role: ROLE_SET_OFFSETS,
            byte_offset: offsets_off,
            byte_len: offsets_len,
            elem_count: (n_docs as u64) + 1,
            width: 4,
            codec: CODEC_FIXED,
            skip_index: Vec::new(),
        },
        ColumnRef {
            name: "set_packed".to_string(),
            role: ROLE_SET_PACKED,
            byte_offset: packed_off,
            byte_len: packed_len,
            elem_count: packed.len() as u64,
            width: 4,
            codec: CODEC_FIXED,
            skip_index: Vec::new(),
        },
        present_column_ref(present_off, present_len, n_words),
        dict_ref,
        postings_ref,
    ];
    finalize_and_write(path, buf, dir)
}

/// Append a var-width column whose entries are arbitrary byte blobs (NOT
/// prefix-similar dictionary strings) under `role`/`name`, reusing the shared
/// LZ4-blocked [`VarColumnWriter`]. The prefix-delta still applies (it just
/// finds `shared == 0` for unrelated blobs, so the suffix is the whole blob);
/// LZ4 then does the real compression. Returns the column's [`ColumnRef`].
fn append_var_blob_column(
    buf: &mut Vec<u8>,
    entries: &[Vec<u8>],
    role: u8,
    name: &str,
) -> Result<ColumnRef> {
    let byte_offset = buf.len() as u64;
    let mut w = VarColumnWriter::new();
    for entry in entries {
        w.push(entry, buf)?;
    }
    let (skip_index, off, byte_len) = w.finish(buf, byte_offset)?;
    Ok(ColumnRef {
        name: name.to_string(),
        role,
        byte_offset: off,
        byte_len,
        elem_count: entries.len() as u64,
        width: 0,
        codec: CODEC_LZ4_VAR,
        skip_index,
    })
}

/// Write a Text segment (Phase 2e-B). Stores the WHOLE inverted text field for
/// `n_docs` docs at `applied_seq`:
///
/// - a sorted token DICT (var-width, [`ROLE_DICT`]) — token `t` has dict-id `t`
///   (BTreeMap iteration is ascending, so the dict is already sorted);
/// - a parallel per-token POSTING-BLOCK column (var-width, [`ROLE_TEXT_POSTINGS`]):
///   the entry at dict-id `t` is token `t`'s delta-varint `(docid_gap, tf)` blob
///   (see [`encode_posting_block`]). Term frequency is NOT rebuildable, so the
///   postings are stored;
/// - a FIXED `u32[n_docs]` DocLen column ([`ROLE_TEXT_DOCLEN`]) = `lens` (0 for
///   a doc with no value), read zero-copy to reproduce `TextIndex::doc_len`;
/// - a present bitset (doc present == `lens[i] > 0`);
/// - the BM25 corpus scalars `doc_count` / `total_doc_len` in the header, so
///   the reader derives the identical `n` / `avgdl`.
///
/// `tokens` is the live `BTreeMap<String, Postings>` (each `Postings` is
/// docid-sorted). `lens[i]` is doc `i`'s length. The reader+writer round-trip
/// the postings exactly, so the sealed BM25 path is bit-identical to the live
/// path.
pub fn write_text_segment(
    path: &Path,
    applied_seq: u64,
    tokens: &std::collections::BTreeMap<String, crate::storage::Postings>,
    lens: &[u32],
    doc_count: u64,
    total_doc_len: u64,
) -> Result<()> {
    let n_docs: u32 = lens
        .len()
        .try_into()
        .context("segment exceeds u32 doc capacity")?;

    // Token dict (ascending — BTreeMap order) and the parallel posting blobs, in
    // the SAME dict-id order so a token's dict index locates its posting block.
    let mut dict: Vec<Vec<u8>> = Vec::with_capacity(tokens.len());
    let mut blobs: Vec<Vec<u8>> = Vec::with_capacity(tokens.len());
    for (tok, postings) in tokens {
        dict.push(tok.as_bytes().to_vec());
        blobs.push(encode_posting_block(postings.docids(), postings.tfs()));
    }

    let mut buf = header_block(applied_seq, n_docs, doc_count, total_doc_len);

    // --- FIXED REGION ---
    // DocLen forward column: u32[n_docs] = lens (0 for an absent doc).
    let doclen_start = page_align(buf.len());
    buf.resize(doclen_start, 0);
    let (doclen_off, doclen_len) = append_u32_column(&mut buf, lens)?;

    // Present bitset: bit i set == doc i has a value (len > 0). The text read
    // paths do not gate on this (doc_len is read directly), but it keeps the
    // layout uniform with every other segment role.
    let present: Vec<bool> = lens.iter().map(|&l| l > 0).collect();
    let (present_off, present_len, n_words) = append_present_bitset(&mut buf, &present)?;

    // --- VAR REGION (after the page-padded fixed region) ---
    let region_end = page_align(buf.len());
    buf.resize(region_end, 0);
    let dict_ref = append_dict_column(&mut buf, &dict)?;
    let postings_ref =
        append_var_blob_column(&mut buf, &blobs, ROLE_TEXT_POSTINGS, "text_postings")?;

    let dir = vec![
        ColumnRef {
            name: "text_doclen".to_string(),
            role: ROLE_TEXT_DOCLEN,
            byte_offset: doclen_off,
            byte_len: doclen_len,
            elem_count: n_docs as u64,
            width: 4,
            codec: CODEC_FIXED,
            skip_index: Vec::new(),
        },
        present_column_ref(present_off, present_len, n_words),
        dict_ref,
        postings_ref,
    ];
    finalize_and_write(path, buf, dir)
}

/// Write the collection-level EID meta segment (Phase 2f-1). `eids[i]` is the
/// external_id string of docid `i` in dense `[0..n_docs)` order — exactly the
/// interner's `to_eid` Vec. Stored as ONE var-width LZ4 column (role
/// [`ROLE_EID`]) by position, so a reopen rebuilds the whole `Interner` from
/// this file alone. No present bitset / fixed region is needed: every docid in
/// `[0..n_docs)` has an external_id (the interner is append-only and dense), so
/// the column's entry count IS `n_docs`. `n_docs` is also stamped in the header
/// for cross-checking. Makes the sealed collection self-describing + reopenable
/// without a CBOR snapshot.
pub fn write_eid_segment(path: &Path, applied_seq: u64, eids: &[&str]) -> Result<()> {
    let n_docs: u32 = eids
        .len()
        .try_into()
        .context("eid segment exceeds u32 doc capacity")?;

    let mut buf = header_block(applied_seq, n_docs, 0, 0);

    // --- VAR REGION (the eid-by-position dictionary). Page-align the start so
    // the var region begins on a clean boundary, mirroring the other writers. ---
    let region_end = page_align(buf.len());
    buf.resize(region_end, 0);
    let entries: Vec<Vec<u8>> = eids.iter().map(|s| s.as_bytes().to_vec()).collect();
    let eid_ref = append_var_blob_column(&mut buf, &entries, ROLE_EID, "eid")?;

    let dir = vec![eid_ref];
    finalize_and_write(path, buf, dir)
}

// ---------------------------------------------------------------------------
// Reader (zero-copy)
// ---------------------------------------------------------------------------

/// A decoded var-column block: the reconstructed full byte strings, in dict-id
/// order. Cached behind an `Arc` so the moka cache hands out cheap clones.
type DecodedBlock = Arc<Vec<Vec<u8>>>;

/// Approximate retained-byte weight of a decoded var block, for the moka
/// byte-budget: the string bytes plus a small per-entry overhead.
fn decoded_block_weight(block: &DecodedBlock) -> u32 {
    let mut w: usize = 32;
    for s in block.iter() {
        w = w.saturating_add(s.len() + 24);
    }
    w.min(u32::MAX as usize) as u32
}

/// Default decompressed-var-block cache budget. The fixed columns are served
/// zero-copy off the mmap and never touch this cache; only the prefix-delta
/// LZ4 dictionary blocks are decoded and cached here. 16 MiB comfortably holds
/// the dictionaries of a typical sealed segment.
const DEFAULT_VAR_CACHE_BYTES: u64 = 16 * 1024 * 1024;

// ---------------------------------------------------------------------------
// BOUNDED DECODED-POSTING CACHE (Phase 2m)
// ---------------------------------------------------------------------------
//
// The decoded-posting accessors (`keyword_postings` / `set_postings` /
// `number_value_postings` / `number_range` / `text_postings`) re-`decode_*`-d a
// FRESH `RoaringBitmap` (or `(docids, tfs)`) per call off the mmap. The forward
// payload is already demand-paged + warmed by the OS page cache, but the
// INVERTED postings had no such hot-zone, so a repeated low-cardinality
// number/range/sort/keyword query paid the whole delta-varint decode every time
// (5x-525x slower than the in-RAM driver that held the bitmaps resident).
//
// This is a BOUNDED, weight-capped cache of DECODED postings — the inverted-index
// analogue of the OS page cache for the forward payload. It holds the RAW
// immutable posting a fresh decode would produce; the `storage.rs` accessors keep
// subtracting the per-field tombstone AFTER the cache fetch, so RESULTS are
// byte-identical. Bounded by a serialized-size weigher + a configurable cap
// (`LUMEN_SEG_POSTING_CACHE_MB`, default 64 MiB), so the 2i scale-proof RSS bound
// still holds (the cap prevents O(cardinality) resident growth); warm => repeated
// queries hit the resident `Arc<RoaringBitmap>` => competitive.
//
// Keyed by a packed `(role, id) -> u64`: `role` distinguishes the column
// (`ROLE_KEYWORD_POSTINGS` / `ROLE_SET_POSTINGS` / `ROLE_NUMBER_POSTINGS`), and
// `id` is the dict-id (Keyword/Set) or the sorted-value index (Number). Both id
// spaces are dense and segment-local, so the pack is collision-free within one
// reader. Text postings carry a parallel `tf` stream, so they use a separate
// `(docids, tfs)` cache keyed by dict-id.

/// A resident decoded docid-only posting (Keyword / Set / Number value). Behind
/// an `Arc` so the moka cache hands out cheap clones — a hit is a refcount bump,
/// not a re-decode.
type CachedPosting = Arc<roaring::RoaringBitmap>;

/// A resident decoded Text posting: the `(docids, tfs)` SoA streams the live
/// `Postings` held. `Arc`-shared like [`CachedPosting`].
type CachedTextPosting = Arc<(Vec<u32>, Vec<u32>)>;

/// Pack a `(role, id)` pair into a collision-free `u64` cache key. `role` is a
/// `ROLE_*` discriminant (small); `id` is a dense segment-local dict-id or
/// sorted-value index (`< 2^32`), so the high byte carries the role and the low
/// 32 bits carry the id with room to spare.
#[inline]
fn posting_cache_key(role: u8, id: u32) -> u64 {
    ((role as u64) << 32) | (id as u64)
}

/// Approximate retained-byte weight of a cached docid posting, for the moka
/// byte-budget. A `RoaringBitmap`'s serialized size is the honest on-heap proxy
/// for its container payload (array/bitset/run blocks); a small constant covers
/// the `Arc` + map-entry overhead so a cache of many tiny postings is still
/// bounded.
fn cached_posting_weight(p: &CachedPosting) -> u32 {
    let bytes = p.serialized_size().saturating_add(64);
    bytes.min(u32::MAX as usize) as u32
}

/// Approximate retained-byte weight of a cached Text posting (`docids` + `tfs`
/// `u32` vectors plus a small constant).
fn cached_text_posting_weight(p: &CachedTextPosting) -> u32 {
    let (docids, tfs) = p.as_ref();
    let bytes = docids
        .len()
        .saturating_add(tfs.len())
        .saturating_mul(4)
        .saturating_add(64);
    bytes.min(u32::MAX as usize) as u32
}

/// Default decoded-posting cache budget (per reader): 64 MiB. Big enough that a
/// warm working set of low-cardinality number/range/sort/keyword postings stays
/// resident (so repeated queries are RAM-speed), small enough that the 2i
/// scale-proof RSS bound still holds (the cap prevents O(cardinality) growth).
/// Overridable via `LUMEN_SEG_POSTING_CACHE_MB`.
const DEFAULT_POSTING_CACHE_BYTES: u64 = 64 * 1024 * 1024;

/// The decoded-posting cache byte budget — `LUMEN_SEG_POSTING_CACHE_MB` (MiB) if
/// set and parseable, else [`DEFAULT_POSTING_CACHE_BYTES`]. A value of `0`
/// disables the cache (max_capacity 0 ⇒ every insert is immediately evicted, so
/// the accessors fall back to a fresh decode — the pre-cache behaviour).
fn posting_cache_bytes() -> u64 {
    match std::env::var("LUMEN_SEG_POSTING_CACHE_MB") {
        Ok(s) => match s.trim().parse::<u64>() {
            Ok(mb) => mb.saturating_mul(1024 * 1024),
            Err(_) => DEFAULT_POSTING_CACHE_BYTES,
        },
        Err(_) => DEFAULT_POSTING_CACHE_BYTES,
    }
}

/// A zero-copy, read-only view over a segment file. `mmap` is the page-aligned
/// kernel mapping; `dir` and the scalar fields are owned (decoded once on
/// open). FIXED columns (Number/Hash/Vector forward, Keyword dict-id, Set CSR
/// offsets + packed) are read zero-copy via `try_cast_slice`. VAR columns (the
/// Keyword/Set string dictionaries) are LZ4-blocked: a block is decompressed on
/// first touch and cached in `block_cache` (a moka byte-weighted cache), keyed
/// by the block's file offset.
///
/// `Send + Sync`: `Arc<Mmap>` + owned fields + `moka::sync::Cache` are all
/// `Send + Sync`; there is no self-referential borrow or `'static` transmute.
pub struct SegmentReader {
    mmap: Arc<memmap2::Mmap>,
    dir: Vec<ColumnRef>,
    applied_seq: u64,
    n_docs: u32,
    /// Phase 2e-B BM25 corpus scalars (0 for a non-text segment): the document
    /// count `N` and the summed document length `Σ|d|`. `avgdl` is derived from
    /// these so the sealed BM25 path uses the identical `n` / `avgdl` the live
    /// `TextIndex` held.
    doc_count: u64,
    total_doc_len: u64,
    /// Decompressed var-block cache, keyed by the block's byte offset in the
    /// file. Byte-weighted to `DEFAULT_VAR_CACHE_BYTES`.
    block_cache: moka::sync::Cache<u64, DecodedBlock>,
    /// BOUNDED decoded-posting cache (Phase 2m): RAW immutable docid-only
    /// postings (Keyword / Set / Number value), keyed by a packed `(role, id)`
    /// (see [`posting_cache_key`]). A hit returns the resident
    /// `Arc<RoaringBitmap>` (refcount bump); a miss decodes the posting block and
    /// inserts. Byte-weighted to [`posting_cache_bytes`] so the 2i RSS bound
    /// holds. The `storage.rs` accessors subtract the per-field tombstone AFTER
    /// the fetch, so cached results stay byte-identical.
    posting_cache: moka::sync::Cache<u64, CachedPosting>,
    /// BOUNDED Text decoded-posting cache (Phase 2m): the `(docids, tfs)` SoA for
    /// a token, keyed by its dict-id. Separate from [`Self::posting_cache`]
    /// because Text carries a parallel `tf` stream the docid-only cache cannot
    /// hold. Same budget + tombstone-after-fetch discipline.
    text_posting_cache: moka::sync::Cache<u64, CachedTextPosting>,
}

impl std::fmt::Debug for SegmentReader {
    /// A terse, allocation-free summary — the mmap bytes and CBOR directory are
    /// not interesting in a `{:?}` dump and would be huge. Lets containers that
    /// hold a `SegmentReader` keep their `#[derive(Debug)]`.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SegmentReader")
            .field("applied_seq", &self.applied_seq)
            .field("n_docs", &self.n_docs)
            .field("columns", &self.dir.len())
            .finish()
    }
}

impl SegmentReader {
    /// Open and validate a segment file. Reads the footer first, validates
    /// `magic2`, the directory crc32, then the header magic / version /
    /// endianness. Any mismatch (bad magic, bad crc, wrong endian, directory
    /// pointer out of range) returns `Err`; the caller discards and replays.
    pub fn open(path: &Path) -> Result<SegmentReader> {
        let file = File::open(path).with_context(|| format!("open {}", path.display()))?;
        // SAFETY: read-only mapping; we never mutate it and treat all bytes as
        // untrusted (bounds-checked, crc-checked before use).
        let mmap = unsafe { memmap2::Mmap::map(&file) }
            .with_context(|| format!("mmap {}", path.display()))?;
        let len = mmap.len();

        // A valid file is at least header + footer.
        if len < HEADER_LEN + FOOTER_LEN {
            bail!("segment too small: {len} bytes");
        }

        // The kernel page-aligns the mapping base; a page-aligned column
        // offset therefore yields an 8-byte-aligned slice for try_cast_slice.
        debug_assert_eq!(
            mmap.as_ptr() as usize % PAGE,
            0,
            "mmap base must be page-aligned"
        );

        // --- FOOTER (tail-first) ---
        let footer_off = len - FOOTER_LEN;
        let footer = Footer::from_bytes(&mmap[footer_off..len])?;
        if footer.magic2 != MAGIC2 {
            bail!("bad footer magic2: {:#x}", footer.magic2);
        }

        // Directory must lie strictly inside [HEADER_LEN, footer_off].
        let dir_offset = footer.dir_offset as usize;
        let dir_len = footer.dir_len as usize;
        let dir_end = dir_offset
            .checked_add(dir_len)
            .ok_or_else(|| anyhow!("directory length overflow"))?;
        if dir_offset < HEADER_LEN || dir_end > footer_off {
            bail!("directory out of range: [{dir_offset}..{dir_end}) vs footer at {footer_off}");
        }

        // --- CRC over the directory bytes ---
        let dir_bytes = &mmap[dir_offset..dir_end];
        let crc = crc32fast::hash(dir_bytes);
        if crc != footer.crc32 {
            bail!(
                "directory crc mismatch: computed {:#x} != footer {:#x}",
                crc,
                footer.crc32
            );
        }

        // --- HEADER ---
        let header = Header::from_bytes(&mmap[..HEADER_LEN])?;
        if header.magic1 != MAGIC1 {
            bail!("bad header magic1: {:#x}", header.magic1);
        }
        if header.format_ver != FORMAT_VER {
            bail!("unsupported format_ver: {}", header.format_ver);
        }
        if header.host_endian_marker != HOST_ENDIAN_MARKER {
            bail!(
                "host endian mismatch: {:#x} != {:#x}",
                header.host_endian_marker,
                HOST_ENDIAN_MARKER
            );
        }

        // --- DIRECTORY (CBOR) ---
        let dir: Vec<ColumnRef> = ciborium::from_reader(dir_bytes)
            .map_err(|e| anyhow!("cbor decode segment directory: {e}"))?;

        let block_cache: moka::sync::Cache<u64, DecodedBlock> = moka::sync::Cache::builder()
            .weigher(|_k: &u64, v: &DecodedBlock| decoded_block_weight(v))
            .max_capacity(DEFAULT_VAR_CACHE_BYTES)
            .build();

        // BOUNDED decoded-posting caches (Phase 2m), mirroring `block_cache`'s
        // byte-weighted moka build. Sized off `LUMEN_SEG_POSTING_CACHE_MB`.
        let posting_budget = posting_cache_bytes();
        let posting_cache: moka::sync::Cache<u64, CachedPosting> = moka::sync::Cache::builder()
            .weigher(|_k: &u64, v: &CachedPosting| cached_posting_weight(v))
            .max_capacity(posting_budget)
            .build();
        let text_posting_cache: moka::sync::Cache<u64, CachedTextPosting> =
            moka::sync::Cache::builder()
                .weigher(|_k: &u64, v: &CachedTextPosting| cached_text_posting_weight(v))
                .max_capacity(posting_budget)
                .build();

        Ok(SegmentReader {
            mmap: Arc::new(mmap),
            dir,
            applied_seq: header.applied_seq,
            n_docs: header.n_docs,
            doc_count: header.doc_count,
            total_doc_len: header.total_doc_len,
            block_cache,
            posting_cache,
            text_posting_cache,
        })
    }

    /// The WAL sequence this segment is current as of.
    pub fn applied_seq(&self) -> u64 {
        self.applied_seq
    }

    /// Number of doc rows in this segment.
    pub fn n_docs(&self) -> u32 {
        self.n_docs
    }

    /// Locate the first directory entry with the given role.
    fn column(&self, role: u8) -> Option<&ColumnRef> {
        self.dir.iter().find(|c| c.role == role)
    }

    /// Borrow a column's bytes, bounds-checked against the mmap. A column ref
    /// pointing past the (possibly truncated) file yields `None` rather than a
    /// panic.
    fn column_bytes(&self, col: &ColumnRef) -> Option<&[u8]> {
        let off = usize::try_from(col.byte_offset).ok()?;
        let len = usize::try_from(col.byte_len).ok()?;
        let end = off.checked_add(len)?;
        self.mmap.get(off..end)
    }

    /// Borrow a FIXED `u32` column's elements zero-copy off the mmap. `None`
    /// for a torn/misaligned column ref — never panics. The Keyword dict-id and
    /// Set offsets/packed columns are all read through this.
    fn u32_column(&self, role: u8) -> Option<&[u32]> {
        let col = self.column(role)?;
        let bytes = self.column_bytes(col)?;
        bytemuck::try_cast_slice::<u8, u32>(bytes).ok()
    }

    /// Decode the per-column skip-index carried inline in a VAR column's
    /// directory entry. `None` if the codec is not LZ4-var or the CBOR is torn.
    fn var_skip_index(&self, col: &ColumnRef) -> Option<SparseVarIndex> {
        if col.codec != CODEC_LZ4_VAR {
            return None;
        }
        ciborium::from_reader(&col.skip_index[..]).ok()
    }

    /// Decompress one var block (or return the cached copy). The block frame is
    /// `u32` compressed-length + LZ4(cbor([`VarBlockBody`])); a truncated /
    /// corrupt frame yields `None` (never panics — the discard-on-torn gate is
    /// the directory crc, so an in-range frame should always decode, but we
    /// honor the no-panic discipline regardless). Caches the reconstructed
    /// strings in the moka byte-weighted cache keyed by `meta.offset`.
    fn decode_var_block(&self, meta: &VarBlockMeta) -> Option<DecodedBlock> {
        if let Some(hit) = self.block_cache.get(&meta.offset) {
            return Some(hit);
        }
        let off = usize::try_from(meta.offset).ok()?;
        // The frame is [u32 len][compressed...]; the len prefix must match the
        // directory's recorded length.
        let len_end = off.checked_add(4)?;
        let len_bytes = self.mmap.get(off..len_end)?;
        let frame_len = u32::from_le_bytes(len_bytes.try_into().ok()?) as usize;
        if frame_len != meta.length as usize {
            return None;
        }
        let comp_end = len_end.checked_add(frame_len)?;
        let compressed = self.mmap.get(len_end..comp_end)?;
        let raw = lz4_flex::decompress_size_prepended(compressed).ok()?;
        let body: VarBlockBody = ciborium::from_reader(&raw[..]).ok()?;
        let decoded: DecodedBlock = Arc::new(body.reconstruct());
        self.block_cache.insert(meta.offset, decoded.clone());
        Some(decoded)
    }

    /// Resolve a dict-id in a VAR dict column to its owning decoded block plus
    /// the in-block position. Binary-searches the skip-index to the rightmost
    /// block with `first_entry <= id`, decompresses it (cache-on-touch), and
    /// bounds-checks the position. `None` for an out-of-range id or a torn
    /// block — never panics.
    fn dict_block_at(&self, dict_role: u8, id: u32) -> Option<(DecodedBlock, usize)> {
        let col = self.column(dict_role)?;
        if id as u64 >= col.elem_count {
            return None;
        }
        let index = self.var_skip_index(col)?;
        let mut lo = 0usize;
        let mut hi = index.blocks.len();
        while lo < hi {
            let mid = (lo + hi) / 2;
            if index.blocks[mid].first_entry <= id {
                lo = mid + 1;
            } else {
                hi = mid;
            }
        }
        if lo == 0 {
            return None;
        }
        let meta = &index.blocks[lo - 1];
        let within = id.checked_sub(meta.first_entry)? as usize;
        if within >= meta.entry_count as usize {
            return None;
        }
        let block = self.decode_var_block(meta)?;
        if within >= block.len() {
            return None;
        }
        Some((block, within))
    }

    /// Fetch a docid-only posting at `(role, id)` through the BOUNDED posting
    /// cache (Phase 2m). On a hit, return the resident `Arc<RoaringBitmap>` (a
    /// refcount bump — no re-decode). On a miss, decode the posting block at this
    /// id (`dict_block_at` + `decode_docid_block`), insert the `Arc`, and return
    /// it. `role` selects the posting column (`ROLE_KEYWORD_POSTINGS` /
    /// `ROLE_SET_POSTINGS` / `ROLE_NUMBER_POSTINGS`); `id` is the dict-id
    /// (Keyword/Set) or sorted-value index (Number). The cached bitmap is the RAW
    /// immutable posting a fresh decode would produce — the `storage.rs` accessor
    /// subtracts the tombstone AFTER this call, so the result stays
    /// byte-identical. `None` for an out-of-range id or a torn block (never
    /// panics; a torn block is never cached).
    fn cached_docid_postings(&self, role: u8, id: u32) -> Option<CachedPosting> {
        let key = posting_cache_key(role, id);
        if let Some(hit) = self.posting_cache.get(&key) {
            return Some(hit);
        }
        let (block, within) = self.dict_block_at(role, id)?;
        let blob = block.get(within)?;
        let docids = decode_docid_block(blob)?;
        let bitmap: CachedPosting = Arc::new(docids.into_iter().collect());
        self.posting_cache.insert(key, bitmap.clone());
        Some(bitmap)
    }

    /// Resolve a dict-id to its decoded UTF-8 string via the [`ROLE_DICT`]
    /// column. `None` for an out-of-range id, a torn block, or non-UTF-8 bytes
    /// (a real dict always holds valid UTF-8 strings). The returned `String` is
    /// owned because the bytes live in the moka-cached decompressed block, not
    /// on the mmap page — see [`Self::keyword_at`].
    fn dict_string(&self, id: u32) -> Option<String> {
        let (block, within) = self.dict_block_at(ROLE_DICT, id)?;
        let bytes = block.get(within)?;
        String::from_utf8(bytes.clone()).ok()
    }

    /// `true` if doc `id` is in range AND its present-bit is set. `false` for
    /// an out-of-range id, an absent doc, or a torn/misaligned present column —
    /// never panics. Every per-doc reader gates on this first.
    fn is_present(&self, id: u32) -> bool {
        if id >= self.n_docs {
            return false;
        }
        let Some(present) = self.column(ROLE_PRESENT) else {
            return false;
        };
        let Some(present_bytes) = self.column_bytes(present) else {
            return false;
        };
        // try_cast_slice (NOT cast_slice): a truncated/misaligned column is an
        // Err we turn into false, never a panic.
        let Ok(present_words) = bytemuck::try_cast_slice::<u8, u64>(present_bytes) else {
            return false;
        };
        let Some(word) = present_words.get((id as usize) / 64) else {
            return false;
        };
        (word >> (id % 64)) & 1 == 1
    }

    /// The decoded `f64` for doc `id`, or `None` if `id` is out of range, the
    /// doc is absent, or any column is torn/misaligned. Never panics.
    pub fn number_at(&self, id: u32) -> Option<f64> {
        if !self.is_present(id) {
            return None;
        }
        // Number forward column: u64 raw bits.
        let number = self.column(ROLE_NUMBER)?;
        let number_bytes = self.column_bytes(number)?;
        let number_words: &[u64] = bytemuck::try_cast_slice(number_bytes).ok()?;
        let bits = number_words.get(id as usize)?;
        Some(f64::from_bits(*bits))
    }

    // -----------------------------------------------------------------------
    // Number SORTED-VALUE range index (Phase 2h-3)
    // -----------------------------------------------------------------------

    /// The ascending `u64[distinct]` sorted-value range index ([`ROLE_NUMBER_SORTED`]),
    /// borrowed zero-copy off the mmap. Each entry is a distinct `SortableF64`
    /// bit key (monotone in numeric order, so the slice is ascending). `None`
    /// when the column is absent (a pre-2h-3 segment) or torn/misaligned — every
    /// range/exact reader gates on this and falls back to `None`/empty, never a
    /// panic. The slice borrows `&self` (it lives on the page).
    fn number_sorted(&self) -> Option<&[u64]> {
        let col = self.column(ROLE_NUMBER_SORTED)?;
        let bytes = self.column_bytes(col)?;
        bytemuck::try_cast_slice::<u8, u64>(bytes).ok()
    }

    /// The number of DISTINCT numeric values in this segment — the length of the
    /// sorted-value column ([`ROLE_NUMBER_SORTED`]). 0 when absent/torn. Mirrors
    /// the in-RAM `NumberIndex.values.len()`. Phase 2h-3.
    pub fn number_distinct_count(&self) -> u64 {
        self.number_sorted().map(|s| s.len() as u64).unwrap_or(0)
    }

    /// Decode the docid-only posting blob at SORTED INDEX `i` (the value's
    /// position in [`ROLE_NUMBER_SORTED`]) into an ascending docid
    /// `RoaringBitmap`. `None` for an out-of-range index or a torn block — never
    /// panics. The Number analogue of `keyword_postings`, but located by the
    /// numeric value's sorted index (the binary-search result), not a string
    /// dict-id. Phase 2h-3.
    fn number_postings_at(&self, i: u32) -> Option<roaring::RoaringBitmap> {
        self.cached_number_postings_at(i).map(|p| (*p).clone())
    }

    /// The cache-resident Number posting at sorted-value index `i` (Phase 2m).
    /// `number_range` / sort-via-sorted-index call this directly to OR the
    /// `Arc<RoaringBitmap>` without an intermediate clone; `number_postings_at`
    /// keeps its owned-bitmap signature for the `number_values_all` enumeration.
    fn cached_number_postings_at(&self, i: u32) -> Option<CachedPosting> {
        self.cached_docid_postings(ROLE_NUMBER_POSTINGS, i)
    }

    /// Binary-search the ascending sorted-value column for `bits`, returning its
    /// index when present, else `None`. The column is the distinct `SortableF64`
    /// bit keys; `bits` is the probe value's `SortableF64.0`. Phase 2h-3.
    fn number_value_index(&self, bits: u64) -> Option<u32> {
        let sorted = self.number_sorted()?;
        match sorted.binary_search(&bits) {
            Ok(i) => u32::try_from(i).ok(),
            Err(_) => None,
        }
    }

    /// EXACT-MATCH: the docids whose number EQUALS the value with `SortableF64`
    /// bit key `bits`, as an ascending docid `RoaringBitmap`. `None` if no
    /// distinct value matches (or the column/block is torn). Byte-identical to
    /// the in-RAM `values.get(&key)` posting at seal. Phase 2h-3.
    pub fn number_value_postings(&self, bits: u64) -> Option<roaring::RoaringBitmap> {
        let i = self.number_value_index(bits)?;
        self.number_postings_at(i)
    }

    /// EXACT-MATCH document frequency: the stored posting length for the value
    /// with bit key `bits`, decoding ONLY the LEB128 `count` prefix (cheap — the
    /// boolean planner's rarest-first selectivity input). `None` if absent.
    /// Phase 2h-3.
    pub fn number_value_df(&self, bits: u64) -> Option<u64> {
        let i = self.number_value_index(bits)?;
        let (block, within) = self.dict_block_at(ROLE_NUMBER_POSTINGS, i)?;
        let blob = block.get(within)?;
        let mut pos = 0usize;
        read_varint(blob, &mut pos)
    }

    /// The half-open `[i_lo, i_hi)` index window into the ascending sorted-value
    /// column selected by the range bounds, honoring inclusivity and open ends.
    /// This is exactly the window `BTreeMap::range((lo, hi))` would walk over the
    /// same total order: the sorted-value `u64` keys are monotone in numeric
    /// order, so `partition_point` over the unsigned keys reproduces the BTreeMap
    /// range bound semantics byte-for-byte (NaN never reaches a key; ±0.0 are
    /// distinct keys handled identically here and in RAM). Phase 2h-3.
    ///
    /// - lo `Some((b, true))`  = `Included(b)` → first index with `k >= b`.
    /// - lo `Some((b, false))` = `Excluded(b)` → first index with `k >  b`.
    /// - lo `None`             = `Unbounded`   → `0`.
    /// - hi `Some((b, true))`  = `Included(b)` → first index with `k >  b`.
    /// - hi `Some((b, false))` = `Excluded(b)` → first index with `k >= b`.
    /// - hi `None`             = `Unbounded`   → `n`.
    fn number_range_window(
        sorted: &[u64],
        lo: Option<(u64, bool)>,
        hi: Option<(u64, bool)>,
    ) -> (usize, usize) {
        let n = sorted.len();
        let i_lo = match lo {
            // Included(b): keep k >= b → first index NOT (k < b).
            Some((b, true)) => sorted.partition_point(|&k| k < b),
            // Excluded(b): keep k > b → first index NOT (k <= b).
            Some((b, false)) => sorted.partition_point(|&k| k <= b),
            None => 0,
        };
        let i_hi = match hi {
            // Included(b): keep k <= b → first index NOT (k <= b).
            Some((b, true)) => sorted.partition_point(|&k| k <= b),
            // Excluded(b): keep k < b → first index NOT (k < b).
            Some((b, false)) => sorted.partition_point(|&k| k < b),
            None => n,
        };
        // A pathological / inverted range yields an empty window, matching an
        // empty BTreeMap range (`hi <= lo`).
        (i_lo, i_hi.max(i_lo))
    }

    /// RANGE: union the docid postings of every distinct value in `[lo, hi)` (per
    /// the inclusive/open-ended bound semantics of [`Self::number_range_window`])
    /// into one ascending `RoaringBitmap`. Binary-searches the sorted-value
    /// column to the lo/hi index bounds (SELECTIVE — it jumps to `i_lo` and scans
    /// to `i_hi`, NOT a full O(n_docs) forward scan), then ORs each in-window
    /// value's posting block. Result is byte-identical to the in-RAM
    /// `values.range((low, high))` posting union at seal. `None` only when the
    /// sorted-value column is absent/torn (a pre-2h-3 segment); an empty window
    /// yields `Some(empty)`. Phase 2h-3.
    pub fn number_range(
        &self,
        lo: Option<(u64, bool)>,
        hi: Option<(u64, bool)>,
    ) -> Option<roaring::RoaringBitmap> {
        let sorted = self.number_sorted()?;
        let (i_lo, i_hi) = Self::number_range_window(sorted, lo, hi);
        let mut acc = roaring::RoaringBitmap::new();
        for i in i_lo..i_hi {
            // Phase 2m: OR the CACHED per-value posting (resident `Arc` on a warm
            // hit) — the in-window values' bitmaps are exactly what the in-RAM
            // `values.range` walk held. Tombstone subtraction stays in
            // `storage.rs` AFTER the union, so the result is byte-identical.
            if let Some(p) = self.cached_number_postings_at(i as u32) {
                acc |= p.as_ref();
            }
        }
        Some(acc)
    }

    /// The number of distinct sorted values plus a per-index cached-posting
    /// accessor are the two primitives the SORT-via-sorted-index walk
    /// (`storage.rs::try_plan`) drives from: it iterates index `0..distinct`
    /// (ascending) or in reverse (descending), reading each value's `SortableF64`
    /// bits ([`Self::number_sorted_bits_at`]) and its cache-resident posting
    /// ([`Self::number_sorted_postings_at`]) IN value order — the disk analogue of
    /// walking the in-RAM `values` BTreeMap, WITHOUT the pre-2m
    /// gather-`number_at`-per-doc + sort. Streaming index-by-index keeps the cache
    /// the only resident structure (no whole-field BTreeMap materialized).

    /// The `SortableF64` bit key at sorted-value index `i` (ascending value
    /// order), or `None` when out of range / column torn. Phase 2m.
    pub fn number_sorted_bits_at(&self, i: u32) -> Option<u64> {
        self.number_sorted()?.get(i as usize).copied()
    }

    /// The cache-resident posting at sorted-value index `i` (Phase 2m) — the
    /// public-to-`storage.rs` name for [`Self::cached_number_postings_at`]. The
    /// posting is the RAW immutable stream; `storage.rs` applies the tombstone via
    /// the per-doc predicate path during the sort walk, so the order + membership
    /// are byte-identical to the in-RAM walk.
    pub fn number_sorted_postings_at(
        &self,
        i: u32,
    ) -> Option<std::sync::Arc<roaring::RoaringBitmap>> {
        self.cached_number_postings_at(i)
    }

    /// RANGE selectivity: the SUM of posting lengths (df) of every distinct value
    /// in `[lo, hi)`, decoding only each value's cheap LEB128 `count` prefix. The
    /// boolean planner's rarest-first cost input for a range conjunct. `None`
    /// when the sorted-value column is absent/torn. Phase 2h-3.
    pub fn number_range_df(&self, lo: Option<(u64, bool)>, hi: Option<(u64, bool)>) -> Option<u64> {
        let sorted = self.number_sorted()?;
        let (i_lo, i_hi) = Self::number_range_window(sorted, lo, hi);
        let mut sum = 0u64;
        for i in i_lo..i_hi {
            if let Some((block, within)) = self.dict_block_at(ROLE_NUMBER_POSTINGS, i as u32) {
                if let Some(blob) = block.get(within) {
                    let mut pos = 0usize;
                    sum += read_varint(blob, &mut pos).unwrap_or(0);
                }
            }
        }
        Some(sum)
    }

    /// Number of distinct numeric values selected by a range. This is a planner
    /// cost primitive: high distinct-window ranges are expensive to materialize
    /// by ORing per-value postings, and can be cheaper as predicates against a
    /// smaller peer bitmap.
    pub fn number_range_distinct_count(
        &self,
        lo: Option<(u64, bool)>,
        hi: Option<(u64, bool)>,
    ) -> Option<u64> {
        let sorted = self.number_sorted()?;
        let (i_lo, i_hi) = Self::number_range_window(sorted, lo, hi);
        Some((i_hi - i_lo) as u64)
    }

    /// The sorted-value index window selected by a range. This is the streaming
    /// primitive for storage's segment-backed standalone range planner: it can walk
    /// only the selected distinct values instead of materializing the whole
    /// `number_values_all()` map.
    pub fn number_range_index_window(
        &self,
        lo: Option<(u64, bool)>,
        hi: Option<(u64, bool)>,
    ) -> Option<(u32, u32)> {
        let sorted = self.number_sorted()?;
        let (i_lo, i_hi) = Self::number_range_window(sorted, lo, hi);
        Some((u32::try_from(i_lo).ok()?, u32::try_from(i_hi).ok()?))
    }

    /// Materialize EVERY distinct numeric value (as its `SortableF64` bit key)
    /// paired with its stored ascending-docid postings, in ascending value
    /// order. `None` if any posting block is torn. The Number analogue of
    /// `keyword_terms_all` / `set_elements_all`: a sealed Number field dropped its
    /// in-RAM `values` driver, so the only way to walk distinct values (for the
    /// segment-aware sorted-iteration / unique-value enumeration) is this on-disk
    /// sorted-value column. The docid stream is byte-identical to what the live
    /// `values[key]` bitmap held at seal. Phase 2h-3.
    pub fn number_values_all(&self) -> Option<Vec<(u64, roaring::RoaringBitmap)>> {
        let sorted = self.number_sorted()?;
        let mut out = Vec::with_capacity(sorted.len());
        for (i, &bits) in sorted.iter().enumerate() {
            let postings = self.number_postings_at(i as u32)?;
            out.push((bits, postings));
        }
        Some(out)
    }

    /// The raw `u64` hash for doc `id`, or `None` if `id` is out of range, the
    /// doc is absent, or the column is torn/misaligned. Never panics. The hash
    /// is stored directly (no transform), so the returned value is bit-equal to
    /// the live forward entry.
    pub fn hash_at(&self, id: u32) -> Option<u64> {
        if !self.is_present(id) {
            return None;
        }
        let hash = self.column(ROLE_HASH)?;
        let hash_bytes = self.column_bytes(hash)?;
        let hash_words: &[u64] = bytemuck::try_cast_slice(hash_bytes).ok()?;
        hash_words.get(id as usize).copied()
    }

    /// A zero-copy borrow of doc `id`'s `dim`-long vector straight off the
    /// mmap, or `None` if `id` is out of range, the doc is absent, the column
    /// is torn/misaligned, or the per-doc window overruns the column. The
    /// returned slice borrows `&self` (it lives on the page), so a flat kNN scan
    /// reads the corpus with no heap copy. Never panics.
    pub fn vector_at(&self, id: u32, dim: usize) -> Option<&[f32]> {
        if dim == 0 || !self.is_present(id) {
            return None;
        }
        let all = self.vectors_slice(dim)?;
        let start = (id as usize).checked_mul(dim)?;
        let end = start.checked_add(dim)?;
        all.get(start..end)
    }

    /// The whole vector forward column as one contiguous `f32[n_docs * dim]`
    /// slice (zero-copy), or `None` if the column is torn/misaligned or its
    /// element count disagrees with `n_docs * dim`. The slice borrows `&self`.
    pub fn vectors_slice(&self, dim: usize) -> Option<&[f32]> {
        if dim == 0 {
            return None;
        }
        let vector = self.column(ROLE_VECTOR)?;
        let vector_bytes = self.column_bytes(vector)?;
        let floats: &[f32] = bytemuck::try_cast_slice(vector_bytes).ok()?;
        // Guard against a directory that disagrees with the header geometry.
        let expect = (self.n_docs as usize).checked_mul(dim)?;
        if floats.len() < expect {
            return None;
        }
        Some(&floats[..expect])
    }

    /// The keyword string for doc `id`, or `None` if `id` is out of range, the
    /// doc has no keyword (present-bit clear or a [`DICT_ABSENT`] dict-id), or
    /// any column is torn — never panics. The dict-id forward column is read
    /// zero-copy off the mmap; the resolved string is materialized from the
    /// moka-cached decompressed dictionary block.
    ///
    /// DEVIATION FROM SPEC (Part A): the brief asks for `keyword_at(id) ->
    /// Option<&str>` "zero-copy off the decompressed dict block". A `&str`
    /// borrowing `&self` is impossible once the decompressed block lives in the
    /// moka cache (the cache, not a `&self` field, owns the bytes — moka hands
    /// out `Arc` clones). The fixed dict-id column IS read zero-copy; only the
    /// final dictionary string is owned. Returning `String` is the minimal,
    /// sound API; the equality / membership compares in `storage.rs` work
    /// identically against an owned `String`.
    pub fn keyword_at(&self, id: u32) -> Option<String> {
        if !self.is_present(id) {
            return None;
        }
        let ids = self.u32_column(ROLE_KEYWORD_DICTID)?;
        let dict_id = *ids.get(id as usize)?;
        if dict_id == DICT_ABSENT {
            return None;
        }
        self.dict_string(dict_id)
    }

    /// The dict-id of keyword `value` in the shared sorted [`ROLE_DICT`]
    /// dictionary, or `None` if absent. Binary-searches the dict-id space
    /// comparing decoded dictionary bytes against `value` (one var block per
    /// probe, cache-on-touch). The Keyword and Text dicts share [`ROLE_DICT`]
    /// and the identical sorted layout, so this delegates to [`Self::text_dict_id`].
    /// Phase 2h-1. Never panics.
    fn keyword_dict_id(&self, value: &str) -> Option<u32> {
        self.text_dict_id(value)
    }

    /// Keyword `value`'s INVERTED posting list as an ascending docid
    /// `RoaringBitmap`, decoded from the parallel [`ROLE_KEYWORD_POSTINGS`]
    /// column located by the value's dict-id. `None` if the value is not in this
    /// segment's dictionary or the posting block is torn — never panics. The
    /// docid stream is byte-identical to what the live `terms[value]` bitmap
    /// held at seal, so the segment-driven Term/Terms/boolean algebra matches
    /// the in-RAM path exactly. Phase 2h-1.
    pub fn keyword_postings(&self, value: &str) -> Option<roaring::RoaringBitmap> {
        let dict_id = self.keyword_dict_id(value)?;
        // Phase 2m: served through the BOUNDED posting cache (resident `Arc` on a
        // warm hit, fresh decode + insert on a miss). The cached bitmap is the RAW
        // immutable posting; `storage.rs::term_postings` subtracts the tombstone
        // AFTER this call, so the result is byte-identical.
        self.cached_docid_postings(ROLE_KEYWORD_POSTINGS, dict_id)
            .map(|p| (*p).clone())
    }

    /// Materialize EVERY keyword term in the [`ROLE_DICT`] string dictionary of a
    /// Keyword segment paired with its stored INVERTED postings, in dict-id
    /// (ascending lexical) order. `None` if any dict entry or posting block is
    /// torn — never panics. Mirrors [`Self::text_tokens_all`] but decodes the
    /// docid-only posting codec ([`decode_docid_block`], no tf). Used by the
    /// segment-aware duplicate / unique-term enumeration (Phase 2h-1 FIX): a
    /// sealed Keyword field dropped its in-RAM `terms` driver, so the only way to
    /// walk distinct values is the on-disk dict. The docid stream is
    /// byte-identical to what the live `terms[value]` bitmap held at seal.
    pub fn keyword_terms_all(&self) -> Option<Vec<(String, roaring::RoaringBitmap)>> {
        let col = self.column(ROLE_DICT)?;
        let n = u32::try_from(col.elem_count).ok()?;
        let mut out = Vec::with_capacity(n as usize);
        for dict_id in 0..n {
            let (block, within) = self.dict_block_at(ROLE_DICT, dict_id)?;
            let term_bytes = block.get(within)?;
            let term = String::from_utf8(term_bytes.clone()).ok()?;
            let (pblock, pwithin) = self.dict_block_at(ROLE_KEYWORD_POSTINGS, dict_id)?;
            let blob = pblock.get(pwithin)?;
            let docids = decode_docid_block(blob)?;
            out.push((term, docids.into_iter().collect()));
        }
        Some(out)
    }

    /// Keyword `value`'s document frequency (`df`) = the stored posting length,
    /// or `None` if the value is absent from the dictionary. Decodes ONLY the
    /// LEB128 `count` prefix of the posting blob (cheap — keeps the boolean
    /// planner's rarest-first clause ordering off the full decode). Mirrors
    /// [`Self::text_token_df`]. Phase 2h-1. Never panics.
    pub fn keyword_df(&self, value: &str) -> Option<u64> {
        let dict_id = self.keyword_dict_id(value)?;
        let (block, within) = self.dict_block_at(ROLE_KEYWORD_POSTINGS, dict_id)?;
        let blob = block.get(within)?;
        let mut pos = 0usize;
        read_varint(blob, &mut pos)
    }

    /// The set members of doc `id` as owned strings (ascending), or `None` if
    /// `id` is out of range, the doc has no set value (present-bit clear), or
    /// any column is torn — never panics. An empty `Some(vec![])` is a
    /// present-but-empty set. The CSR offsets + packed dict-id columns are read
    /// zero-copy off the mmap; only the dictionary strings are materialized
    /// (from the moka-cached blocks). See [`Self::keyword_at`] for why the
    /// strings are owned rather than `&str`.
    pub fn set_at(&self, id: u32) -> Option<Vec<String>> {
        if !self.is_present(id) {
            return None;
        }
        let offsets = self.u32_column(ROLE_SET_OFFSETS)?;
        let packed = self.u32_column(ROLE_SET_PACKED)?;
        // CSR: doc i's members are packed[offsets[i]..offsets[i+1]]. Guard the
        // off-by-one — offsets has n_docs+1 entries, so offsets[id+1] exists.
        let lo = *offsets.get(id as usize)? as usize;
        let hi = *offsets.get(id as usize + 1)? as usize;
        if hi < lo || hi > packed.len() {
            return None;
        }
        let mut out = Vec::with_capacity(hi - lo);
        for &dict_id in &packed[lo..hi] {
            // A torn dict entry aborts the whole doc rather than silently
            // dropping a member (a partial member set would be a wrong answer).
            out.push(self.dict_string(dict_id)?);
        }
        Some(out)
    }

    /// The dict-id of set element `value` in the shared sorted [`ROLE_DICT`]
    /// dictionary, or `None` if absent. The Set, Keyword and Text dicts all
    /// share [`ROLE_DICT`] and the identical sorted layout, so this delegates to
    /// [`Self::text_dict_id`]. Phase 2h-2. Never panics.
    fn set_dict_id(&self, value: &str) -> Option<u32> {
        self.text_dict_id(value)
    }

    /// Set element `value`'s INVERTED posting list (the docids whose set
    /// contains `value`) as an ascending docid `RoaringBitmap`, decoded from the
    /// parallel [`ROLE_SET_POSTINGS`] column located by the value's dict-id.
    /// `None` if the value is not in this segment's dictionary or the posting
    /// block is torn — never panics. The docid stream is byte-identical to what
    /// the live `elements[value]` bitmap held at seal, so the segment-driven
    /// membership / Terms / boolean algebra matches the in-RAM path exactly.
    /// Phase 2h-2. The Set analogue of [`Self::keyword_postings`].
    pub fn set_postings(&self, value: &str) -> Option<roaring::RoaringBitmap> {
        let dict_id = self.set_dict_id(value)?;
        // Phase 2m: served through the BOUNDED posting cache, same RAW-immutable +
        // tombstone-after-fetch discipline as `keyword_postings`.
        self.cached_docid_postings(ROLE_SET_POSTINGS, dict_id)
            .map(|p| (*p).clone())
    }

    /// Set element `value`'s document frequency (`df`) = the stored posting
    /// length, or `None` if the value is absent from the dictionary. Decodes
    /// ONLY the LEB128 `count` prefix of the posting blob (cheap — keeps the
    /// boolean planner's rarest-first clause ordering off the full decode).
    /// Phase 2h-2. The Set analogue of [`Self::keyword_df`]. Never panics.
    pub fn set_df(&self, value: &str) -> Option<u64> {
        let dict_id = self.set_dict_id(value)?;
        let (block, within) = self.dict_block_at(ROLE_SET_POSTINGS, dict_id)?;
        let blob = block.get(within)?;
        let mut pos = 0usize;
        read_varint(blob, &mut pos)
    }

    /// Materialize EVERY set element in the [`ROLE_DICT`] string dictionary of a
    /// Set segment paired with its stored INVERTED postings, in dict-id
    /// (ascending lexical) order. `None` if any dict entry or posting block is
    /// torn — never panics. The Set analogue of [`Self::keyword_terms_all`]:
    /// used by the segment-aware duplicate / unique-element enumeration (Phase
    /// 2h-2 FIX), since a sealed Set field dropped its in-RAM `elements` driver
    /// and the only way to walk distinct values is the on-disk dict. The docid
    /// stream is byte-identical to what the live `elements[value]` bitmap held
    /// at seal.
    pub fn set_elements_all(&self) -> Option<Vec<(String, roaring::RoaringBitmap)>> {
        let col = self.column(ROLE_DICT)?;
        let n = u32::try_from(col.elem_count).ok()?;
        let mut out = Vec::with_capacity(n as usize);
        for dict_id in 0..n {
            let (block, within) = self.dict_block_at(ROLE_DICT, dict_id)?;
            let el_bytes = block.get(within)?;
            let el = String::from_utf8(el_bytes.clone()).ok()?;
            let (pblock, pwithin) = self.dict_block_at(ROLE_SET_POSTINGS, dict_id)?;
            let blob = pblock.get(pwithin)?;
            let docids = decode_docid_block(blob)?;
            out.push((el, docids.into_iter().collect()));
        }
        Some(out)
    }

    // -----------------------------------------------------------------------
    // Text segment reads (Phase 2e-B)
    // -----------------------------------------------------------------------

    /// The dict-id of `token` in the [`ROLE_DICT`] token dictionary, or `None`
    /// if the token is absent. The dict is sorted ascending (BTreeMap order at
    /// write), so this binary-searches the dict-id space comparing the decoded
    /// dictionary bytes against `token`. Each probe decodes one var block
    /// (cache-on-touch). Never panics.
    fn text_dict_id(&self, token: &str) -> Option<u32> {
        let col = self.column(ROLE_DICT)?;
        let n = u32::try_from(col.elem_count).ok()?;
        let needle = token.as_bytes();
        let mut lo = 0u32;
        let mut hi = n; // exclusive
        while lo < hi {
            let mid = lo + (hi - lo) / 2;
            let (block, within) = self.dict_block_at(ROLE_DICT, mid)?;
            let entry = block.get(within)?;
            match entry.as_slice().cmp(needle) {
                std::cmp::Ordering::Less => lo = mid + 1,
                std::cmp::Ordering::Greater => hi = mid,
                std::cmp::Ordering::Equal => return Some(mid),
            }
        }
        None
    }

    /// Token `token`'s stored postings as `(docids, tfs)`, docid-ascending — the
    /// exact streams the live `Postings` SoA holds. `None` if the token is not
    /// in this segment's dictionary, or the posting block is torn. The fixed
    /// docid/tf streams are decoded from the token's delta-varint blob, located
    /// by its dict-id in the parallel [`ROLE_TEXT_POSTINGS`] column. Never panics.
    pub fn text_postings(&self, token: &str) -> Option<(Vec<u32>, Vec<u32>)> {
        let cached = self.text_postings_arc(token)?;
        Some((cached.0.clone(), cached.1.clone()))
    }

    /// The cache-resident Text posting `Arc<(docids, tfs)>` for `token` (Phase
    /// 2m). A warm hit returns the shared `Arc` (a refcount bump — NO vector
    /// copy); a miss decodes the posting block, inserts, and returns the `Arc`.
    /// `storage.rs::tok_postings` holds this `Arc` directly so a per-candidate
    /// BM25 `match_doc_score` probe (`.tf(id)` binary-search) NEVER re-decodes or
    /// re-clones the whole posting — the cure for the `filtered_search` 25x disk
    /// blow-up where the previous owned-clone path copied the entire ~25k-docid
    /// posting on every candidate doc. The streams are the RAW immutable postings;
    /// `tok_postings` filters the tombstone into its own owned copy only when a
    /// delete is pending (the rare path). `text_postings` keeps its owned-tuple
    /// signature for callers that need to mutate (re-seal).
    pub fn text_postings_arc(&self, token: &str) -> Option<std::sync::Arc<(Vec<u32>, Vec<u32>)>> {
        let dict_id = self.text_dict_id(token)?;
        let key = posting_cache_key(ROLE_TEXT_POSTINGS, dict_id);
        if let Some(hit) = self.text_posting_cache.get(&key) {
            return Some(hit);
        }
        let (block, within) = self.dict_block_at(ROLE_TEXT_POSTINGS, dict_id)?;
        let blob = block.get(within)?;
        let (docids, tfs) = decode_posting_block(blob)?;
        let cached: CachedTextPosting = Arc::new((docids, tfs));
        self.text_posting_cache.insert(key, cached.clone());
        Some(cached)
    }

    /// Doc `id`'s BM25 length from the fixed [`ROLE_TEXT_DOCLEN`] column, read
    /// zero-copy off the mmap. Reproduces `TextIndex::doc_len(id)`: the stored
    /// `lens[id]`, or 0 when `id` is out of range or the column is torn — never
    /// panics. Does NOT gate on the present bitset (a 0-length present doc and an
    /// out-of-range doc are both length 0, matching the live `unwrap_or(0)`).
    pub fn text_doc_len(&self, id: u32) -> u32 {
        let Some(doclen) = self.u32_column(ROLE_TEXT_DOCLEN) else {
            return 0;
        };
        doclen.get(id as usize).copied().unwrap_or(0)
    }

    /// Borrow the whole text doc-length column. Hot BM25 paths use this to avoid
    /// re-resolving the fixed column for every scored docid.
    pub fn text_doc_lens(&self) -> Option<&[u32]> {
        self.u32_column(ROLE_TEXT_DOCLEN)
    }

    /// Token `token`'s document frequency (`df`) = the stored posting length,
    /// or 0 if the token is absent. Decodes only the LEB128 `count` prefix of
    /// the posting blob (cheap). Never panics.
    pub fn text_token_df(&self, token: &str) -> usize {
        let Some(dict_id) = self.text_dict_id(token) else {
            return 0;
        };
        let Some((block, within)) = self.dict_block_at(ROLE_TEXT_POSTINGS, dict_id) else {
            return 0;
        };
        let Some(blob) = block.get(within) else {
            return 0;
        };
        let mut pos = 0usize;
        read_varint(blob, &mut pos).unwrap_or(0) as usize
    }

    /// The BM25 corpus document count `N` carried in the header (0 for a
    /// non-text segment).
    pub fn text_doc_count(&self) -> u64 {
        self.doc_count
    }

    /// The BM25 corpus summed document length `Σ|d|` carried in the header
    /// (0 for a non-text segment).
    pub fn text_total_doc_len(&self) -> u64 {
        self.total_doc_len
    }

    // -----------------------------------------------------------------------
    // Collection EID column (Phase 2f-1)
    // -----------------------------------------------------------------------

    /// The external_id string for docid `id`, read from the [`ROLE_EID`]
    /// by-position dictionary column, or `None` if `id` is out of range, the
    /// column is absent (not an eid meta segment), or its block is torn /
    /// non-UTF-8 — never panics. The string is owned (it lives in the
    /// moka-cached decompressed block, not on the mmap page; see
    /// [`Self::keyword_at`] for why var-column reads can't borrow `&self`).
    pub fn eid_at(&self, id: u32) -> Option<String> {
        let (block, within) = self.dict_block_at(ROLE_EID, id)?;
        let bytes = block.get(within)?;
        String::from_utf8(bytes.clone()).ok()
    }

    /// The logical entry count of the [`ROLE_EID`] column — the number of
    /// external_ids stored, i.e. the collection's dense docid count. 0 if this
    /// is not an eid meta segment.
    pub fn eid_count(&self) -> u32 {
        self.column(ROLE_EID)
            .and_then(|c| u32::try_from(c.elem_count).ok())
            .unwrap_or(0)
    }

    /// Materialize every external_id in dense docid order `[0..eid_count)`.
    /// `None` if any entry is torn (a partial interner would be a wrong
    /// rebuild, so the whole reopen aborts rather than silently dropping a
    /// doc). Used by `Collection::open_from_segments` to rebuild the
    /// `Interner`.
    pub fn eids_all(&self) -> Option<Vec<String>> {
        let n = self.eid_count();
        let mut out = Vec::with_capacity(n as usize);
        for id in 0..n {
            out.push(self.eid_at(id)?);
        }
        Some(out)
    }

    /// Materialize every token in the [`ROLE_DICT`] token dictionary of a Text
    /// segment paired with its stored postings, in dict-id (ascending lexical)
    /// order. `None` if any dict entry or posting block is torn. Used to
    /// reconstruct the live `tokens` BTreeMap on reopen (and for a CBOR snapshot
    /// taken after a Text seal, where the in-RAM `tokens` was dropped). The
    /// `(docids, tfs)` streams are byte-identical to what the live index held at
    /// seal.
    pub fn text_tokens_all(&self) -> Option<Vec<(String, Vec<u32>, Vec<u32>)>> {
        let col = self.column(ROLE_DICT)?;
        let n = u32::try_from(col.elem_count).ok()?;
        let mut out = Vec::with_capacity(n as usize);
        for dict_id in 0..n {
            let (block, within) = self.dict_block_at(ROLE_DICT, dict_id)?;
            let tok_bytes = block.get(within)?;
            let tok = String::from_utf8(tok_bytes.clone()).ok()?;
            let (pblock, pwithin) = self.dict_block_at(ROLE_TEXT_POSTINGS, dict_id)?;
            let blob = pblock.get(pwithin)?;
            let (docids, tfs) = decode_posting_block(blob)?;
            out.push((tok, docids, tfs));
        }
        Some(out)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Seek, SeekFrom};

    /// Unique temp path per test, cleaned up by the OS temp dir convention.
    fn tmp_path(tag: &str) -> std::path::PathBuf {
        let dir =
            std::env::temp_dir().join(format!("lumen-segment-{}-{}", std::process::id(), tag));
        std::fs::create_dir_all(&dir).unwrap();
        dir.join("col.lseg")
    }

    #[test]
    fn round_trip_small() {
        let path = tmp_path("round-trip-small");
        let values = vec![Some(1.5), None, Some(-3.0), Some(0.0)];
        write_number_segment(&path, 42, &values).unwrap();

        let r = SegmentReader::open(&path).unwrap();
        assert_eq!(r.applied_seq(), 42);
        assert_eq!(r.n_docs(), 4);
        assert_eq!(r.number_at(0), Some(1.5));
        assert_eq!(r.number_at(1), None); // absent doc
        assert_eq!(r.number_at(2), Some(-3.0));
        assert_eq!(r.number_at(3), Some(0.0));
        assert_eq!(r.number_at(4), None); // id >= n_docs
        assert_eq!(r.number_at(u32::MAX), None);

        std::fs::remove_file(&path).ok();
    }

    /// Same `sortable_bits` transform `storage::SortableF64::new` applies, so the
    /// segment tests build probe keys identically to the runtime.
    fn bits(x: f64) -> u64 {
        sortable_bits(x)
    }

    /// Phase 2h-3: the SORTED-VALUE range index round-trips — distinct ascending
    /// values, exact-match postings + df, range postings honoring every bound,
    /// and the distinct-count. Values include negatives, ±0.0, and a duplicate.
    #[test]
    fn number_sorted_range_round_trip() {
        let path = tmp_path("number-sorted-range");
        // docids:        0     1      2     3    4     5
        let values = vec![
            Some(5.0),
            None,
            Some(-3.0),
            Some(0.0),
            Some(5.0),
            Some(-3.0),
        ];
        write_number_segment(&path, 1, &values).unwrap();
        let r = SegmentReader::open(&path).unwrap();

        // Distinct ascending values: -3.0, 0.0, 5.0.
        assert_eq!(r.number_distinct_count(), 3);

        // Exact-match postings (ascending docids) + df.
        assert_eq!(
            r.number_value_postings(bits(-3.0)),
            Some([2u32, 5].into_iter().collect())
        );
        assert_eq!(
            r.number_value_postings(bits(0.0)),
            Some([3u32].into_iter().collect())
        );
        assert_eq!(
            r.number_value_postings(bits(5.0)),
            Some([0u32, 4].into_iter().collect())
        );
        assert_eq!(r.number_value_postings(bits(99.0)), None); // absent value
        assert_eq!(r.number_value_df(bits(-3.0)), Some(2));
        assert_eq!(r.number_value_df(bits(0.0)), Some(1));
        assert_eq!(r.number_value_df(bits(5.0)), Some(2));
        assert_eq!(r.number_value_df(bits(99.0)), None);

        // RANGE bound semantics. all = {-3.0:[2,5], 0.0:[3], 5.0:[0,4]}.
        let all: roaring::RoaringBitmap = [0u32, 2, 3, 4, 5].into_iter().collect();
        // fully open
        assert_eq!(r.number_range(None, None), Some(all.clone()));
        // [-3.0, 5.0] inclusive both → all
        assert_eq!(
            r.number_range(Some((bits(-3.0), true)), Some((bits(5.0), true))),
            Some(all.clone())
        );
        // (-3.0, 5.0) exclusive both → only 0.0 → {3}
        assert_eq!(
            r.number_range(Some((bits(-3.0), false)), Some((bits(5.0), false))),
            Some([3u32].into_iter().collect())
        );
        // [-3.0, 5.0) → -3.0, 0.0 → {2,5,3}
        assert_eq!(
            r.number_range(Some((bits(-3.0), true)), Some((bits(5.0), false))),
            Some([2u32, 3, 5].into_iter().collect())
        );
        // (-3.0, 5.0] → 0.0, 5.0 → {3,0,4}
        assert_eq!(
            r.number_range(Some((bits(-3.0), false)), Some((bits(5.0), true))),
            Some([0u32, 3, 4].into_iter().collect())
        );
        // open-low (.. 0.0]  → -3.0, 0.0 → {2,5,3}
        assert_eq!(
            r.number_range(None, Some((bits(0.0), true))),
            Some([2u32, 3, 5].into_iter().collect())
        );
        // open-low (.. 0.0)  → -3.0 → {2,5}
        assert_eq!(
            r.number_range(None, Some((bits(0.0), false))),
            Some([2u32, 5].into_iter().collect())
        );
        // open-high [0.0 ..) → 0.0, 5.0 → {3,0,4}
        assert_eq!(
            r.number_range(Some((bits(0.0), true)), None),
            Some([0u32, 3, 4].into_iter().collect())
        );
        // open-high (0.0 ..) → 5.0 → {0,4}
        assert_eq!(
            r.number_range(Some((bits(0.0), false)), None),
            Some([0u32, 4].into_iter().collect())
        );
        // exact via inclusive lo==hi → single value 0.0
        assert_eq!(
            r.number_range(Some((bits(0.0), true)), Some((bits(0.0), true))),
            Some([3u32].into_iter().collect())
        );
        // empty: exclusive lo==hi
        assert_eq!(
            r.number_range(Some((bits(0.0), false)), Some((bits(0.0), false))),
            Some(roaring::RoaringBitmap::new())
        );
        // empty: inverted lo > hi
        assert_eq!(
            r.number_range(Some((bits(5.0), true)), Some((bits(-3.0), true))),
            Some(roaring::RoaringBitmap::new())
        );

        std::fs::remove_file(&path).ok();
    }

    /// The SORTED-VALUE range index must reproduce a BTreeMap range walk
    /// byte-identically over a LARGE, multi-block corpus, across every bound
    /// shape — the on-disk binary-search == the in-RAM `values.range`.
    #[test]
    fn number_range_matches_btreemap_oracle_multi_block() {
        let path = tmp_path("number-range-oracle");
        // 8k docs over ~2k distinct values → the posting var-column spans several
        // 64KB LZ4 blocks; the sorted-value column has ~2k entries.
        let n = 8_000usize;
        let values: Vec<Option<f64>> = (0..n)
            .map(|i| {
                if i % 37 == 0 {
                    None
                } else {
                    // Spread across negatives and positives with duplicates.
                    Some(((i % 2000) as f64 - 1000.0) * 0.5)
                }
            })
            .collect();
        write_number_segment(&path, 1, &values).unwrap();
        let r = SegmentReader::open(&path).unwrap();

        // Build the in-RAM oracle: BTreeMap<bit-key, ascending docids>.
        use std::collections::BTreeMap;
        let mut oracle: BTreeMap<u64, roaring::RoaringBitmap> = BTreeMap::new();
        for (id, v) in values.iter().enumerate() {
            if let Some(x) = v {
                oracle.entry(bits(*x)).or_default().insert(id as u32);
            }
        }
        assert_eq!(r.number_distinct_count(), oracle.len() as u64);

        // A range of probe values, including endpoints sitting exactly on keys.
        let probes = [-1000.0, -500.0, -3.0, -0.5, 0.0, 0.5, 250.0, 499.5, 1000.0];
        for &lo in &probes {
            for &hi in &probes {
                for &lo_incl in &[true, false] {
                    for &hi_incl in &[true, false] {
                        use std::ops::Bound;
                        let lo_b = bits(lo);
                        let hi_b = bits(hi);
                        // BTreeMap::range PANICS on an inverted / degenerate-exclusive
                        // pair; the on-disk window collapses it to empty. Compare the
                        // disk's empty result against the KNOWN-empty oracle without
                        // calling the panicking `oracle.range`.
                        let empty_pair = lo_b > hi_b || (lo_b == hi_b && (!lo_incl || !hi_incl));
                        let got = r
                            .number_range(Some((lo_b, lo_incl)), Some((hi_b, hi_incl)))
                            .unwrap();
                        if empty_pair {
                            assert!(
                                got.is_empty(),
                                "inverted/degenerate range [{lo} incl={lo_incl}, {hi} incl={hi_incl}) must be empty"
                            );
                            continue;
                        }
                        let low = if lo_incl {
                            Bound::Included(lo_b)
                        } else {
                            Bound::Excluded(lo_b)
                        };
                        let high = if hi_incl {
                            Bound::Included(hi_b)
                        } else {
                            Bound::Excluded(hi_b)
                        };
                        let mut want = roaring::RoaringBitmap::new();
                        for (_, set) in oracle.range((low, high)) {
                            want |= set;
                        }
                        assert_eq!(
                            got, want,
                            "range [{lo} incl={lo_incl}, {hi} incl={hi_incl}) diverged from BTreeMap oracle"
                        );
                    }
                }
            }
        }
        // Open-ended probes vs the oracle's half-open / unbounded ranges.
        for &p in &probes {
            for &incl in &[true, false] {
                use std::ops::Bound;
                let b = if incl {
                    Bound::Included(bits(p))
                } else {
                    Bound::Excluded(bits(p))
                };
                // open-high [p.. / (p..
                let mut want_hi = roaring::RoaringBitmap::new();
                for (_, s) in oracle.range((b, Bound::Unbounded)) {
                    want_hi |= s;
                }
                assert_eq!(
                    r.number_range(Some((bits(p), incl)), None).unwrap(),
                    want_hi,
                    "open-high diverged at {p}"
                );
                // open-low ..p] / ..p)
                let mut want_lo = roaring::RoaringBitmap::new();
                for (_, s) in oracle.range((Bound::Unbounded, b)) {
                    want_lo |= s;
                }
                assert_eq!(
                    r.number_range(None, Some((bits(p), incl))).unwrap(),
                    want_lo,
                    "open-low diverged at {p}"
                );
            }
        }

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn crc_catches_directory_corruption() {
        let path = tmp_path("crc-corruption");
        let values = vec![Some(1.0), Some(2.0), Some(3.0)];
        write_number_segment(&path, 7, &values).unwrap();

        // Read the footer to find where the directory lives, then flip one
        // byte inside the directory region.
        let mut bytes = std::fs::read(&path).unwrap();
        let len = bytes.len();
        let footer = Footer::from_bytes(&bytes[len - FOOTER_LEN..len]).unwrap();
        let dir_off = footer.dir_offset as usize;
        // Flip a byte at the start of the directory region.
        bytes[dir_off] ^= 0xFF;
        std::fs::write(&path, &bytes).unwrap();

        let err = SegmentReader::open(&path);
        assert!(err.is_err(), "crc must reject directory corruption");

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn truncated_column_never_panics() {
        let path = tmp_path("truncated-column");
        // Large enough that the number column is non-trivial.
        let values: Vec<Option<f64>> = (0..2000).map(|i| Some(i as f64)).collect();
        write_number_segment(&path, 9, &values).unwrap();

        // Truncate the file to the middle of the fixed-width region (after the
        // header, well before the directory/footer). open() should reject it
        // (footer/dir gone); if it somehow opens, number_at must not panic.
        let full = std::fs::metadata(&path).unwrap().len();
        let truncate_to = HEADER_LEN as u64 + 64; // mid-column, no footer
        assert!(truncate_to < full);
        {
            let f = std::fs::OpenOptions::new().write(true).open(&path).unwrap();
            f.set_len(truncate_to).unwrap();
        }

        match SegmentReader::open(&path) {
            Ok(r) => {
                // Must not panic for any id.
                for id in 0..3000u32 {
                    let _ = r.number_at(id);
                }
            }
            Err(_) => { /* expected: torn file rejected */ }
        }

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn truncated_mid_column_with_intact_footer_returns_none() {
        // A subtler torn case: keep the footer/directory addressable but make a
        // column ref point past the actual file end by shrinking the file from
        // the middle is impossible without breaking the footer, so instead we
        // forge a directory whose number column overruns the file. We do that
        // by truncating just the trailing zero-pad + tail so the column len in
        // the directory exceeds the available bytes. Simplest robust check:
        // open a valid file, then truncate so the mmap is shorter than a
        // column ref claims, and confirm column_bytes()->None path.
        let path = tmp_path("mid-column-none");
        let values: Vec<Option<f64>> = (0..1000).map(|i| Some(i as f64)).collect();
        write_number_segment(&path, 3, &values).unwrap();

        // Read full file, then rebuild it shorter than the number column needs
        // while preserving a self-consistent footer+directory pointing at the
        // ORIGINAL (now out-of-range) offsets. We do this by chopping bytes out
        // of the middle of the number column and re-appending the original
        // directory + footer, so dir crc still matches but the column overruns.
        let original = std::fs::read(&path).unwrap();
        let len = original.len();
        let footer = Footer::from_bytes(&original[len - FOOTER_LEN..len]).unwrap();
        let dir_off = footer.dir_offset as usize;
        let dir_end = dir_off + footer.dir_len as usize;

        // New file = header + a too-short fixed region + original dir + footer.
        // Keep only HEADER_LEN + 16 bytes of the fixed region (number column
        // ref will claim far more). Then append the unchanged directory bytes
        // and footer, but rewrite the footer's dir_offset to the new location.
        let mut forged = Vec::new();
        forged.extend_from_slice(&original[..HEADER_LEN + 16]); // tiny fixed region
        let new_dir_off = forged.len() as u64;
        forged.extend_from_slice(&original[dir_off..dir_end]); // same dir bytes => same crc
        let new_footer = Footer {
            dir_offset: new_dir_off,
            dir_len: footer.dir_len,
            crc32: footer.crc32,
            magic2: footer.magic2,
        };
        forged.extend_from_slice(&new_footer.to_bytes());
        std::fs::write(&path, &forged).unwrap();

        // open() succeeds (header+footer+dir are valid), but the number column
        // ref overruns the file, so number_at must return None, never panic.
        match SegmentReader::open(&path) {
            Ok(r) => {
                for id in 0..1500u32 {
                    let _ = r.number_at(id); // must not panic
                }
                // id 0 lives in the surviving 16 bytes? number col starts page
                // aligned at 4096, region is only 4096+16, so column_bytes for
                // the full claimed length is None => number_at None.
                assert_eq!(r.number_at(0), None);
            }
            Err(_) => { /* also acceptable */ }
        }

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn bad_magic_rejected() {
        let path = tmp_path("bad-magic");
        let values = vec![Some(1.0), Some(2.0)];
        write_number_segment(&path, 1, &values).unwrap();

        // Corrupt magic2 (the last 4 bytes of the file).
        let mut f = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&path)
            .unwrap();
        let len = f.metadata().unwrap().len();
        f.seek(SeekFrom::Start(len - 4)).unwrap();
        let mut m = [0u8; 4];
        f.read_exact(&mut m).unwrap();
        f.seek(SeekFrom::Start(len - 4)).unwrap();
        f.write_all(&[m[0] ^ 0xFF, m[1], m[2], m[3]]).unwrap();
        drop(f);

        assert!(
            SegmentReader::open(&path).is_err(),
            "bad magic2 must reject"
        );

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn large_round_trip_page_alignment() {
        let path = tmp_path("large-round-trip");
        let n = 10_000usize;
        // Present pattern: every 3rd doc absent.
        let values: Vec<Option<f64>> = (0..n)
            .map(|i| {
                if i % 3 == 0 {
                    None
                } else {
                    Some(i as f64 * 0.25)
                }
            })
            .collect();
        write_number_segment(&path, 123_456, &values).unwrap();

        let r = SegmentReader::open(&path).unwrap();
        assert_eq!(r.applied_seq(), 123_456);
        assert_eq!(r.n_docs(), n as u32);
        for (i, v) in values.iter().enumerate() {
            assert_eq!(r.number_at(i as u32), *v, "mismatch at id {i}");
        }
        assert_eq!(r.number_at(n as u32), None);

        std::fs::remove_file(&path).ok();
    }

    /// Compile-time assertion that the reader is Send + Sync (the disk tier
    /// shares it across serving threads).
    #[test]
    fn reader_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<SegmentReader>();
    }

    #[test]
    fn hash_round_trip_small() {
        let path = tmp_path("hash-round-trip-small");
        let values = vec![Some(0xDEAD_BEEFu64), None, Some(0), Some(u64::MAX)];
        write_hash_segment(&path, 11, &values).unwrap();

        let r = SegmentReader::open(&path).unwrap();
        assert_eq!(r.applied_seq(), 11);
        assert_eq!(r.n_docs(), 4);
        assert_eq!(r.hash_at(0), Some(0xDEAD_BEEF));
        assert_eq!(r.hash_at(1), None); // absent
        assert_eq!(r.hash_at(2), Some(0)); // present-and-zero != absent
        assert_eq!(r.hash_at(3), Some(u64::MAX));
        assert_eq!(r.hash_at(4), None); // id >= n_docs
                                        // Cross-role: a hash segment has no Number column.
        assert_eq!(r.number_at(0), None);

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn hash_large_round_trip() {
        let path = tmp_path("hash-large");
        let n = 5000usize;
        let values: Vec<Option<u64>> = (0..n)
            .map(|i| {
                if i % 4 == 0 {
                    None
                } else {
                    Some((i as u64).wrapping_mul(0x9E37_79B9))
                }
            })
            .collect();
        write_hash_segment(&path, 7, &values).unwrap();
        let r = SegmentReader::open(&path).unwrap();
        for (i, v) in values.iter().enumerate() {
            assert_eq!(r.hash_at(i as u32), *v, "mismatch at {i}");
        }
        assert_eq!(r.hash_at(n as u32), None);
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn vector_round_trip_small() {
        let path = tmp_path("vector-round-trip-small");
        let dim = 3;
        let v0 = [1.0f32, -2.0, 0.5];
        let v2 = [3.5f32, 4.0, -1.25];
        let vectors: Vec<Option<&[f32]>> = vec![Some(&v0[..]), None, Some(&v2[..])];
        write_vector_segment(&path, 5, dim, &vectors).unwrap();

        let r = SegmentReader::open(&path).unwrap();
        assert_eq!(r.applied_seq(), 5);
        assert_eq!(r.n_docs(), 3);
        assert_eq!(r.vector_at(0, dim), Some(&v0[..]));
        assert_eq!(r.vector_at(1, dim), None); // absent
        assert_eq!(r.vector_at(2, dim), Some(&v2[..]));
        assert_eq!(r.vector_at(3, dim), None); // id >= n_docs

        // The whole column is dense (absent doc is dim zeros).
        let all = r.vectors_slice(dim).unwrap();
        assert_eq!(all.len(), 3 * dim);
        assert_eq!(&all[0..3], &v0);
        assert_eq!(&all[3..6], &[0.0, 0.0, 0.0]); // absent doc = zeros
        assert_eq!(&all[6..9], &v2);

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn vector_bits_are_exact() {
        // Bit-exactness across awkward float values (incl negative zero / inf).
        let path = tmp_path("vector-bits");
        let dim = 4;
        let v: [f32; 4] = [f32::MIN_POSITIVE, -0.0, f32::INFINITY, 1.0 / 3.0];
        let vectors: Vec<Option<&[f32]>> = vec![Some(&v[..])];
        write_vector_segment(&path, 1, dim, &vectors).unwrap();
        let r = SegmentReader::open(&path).unwrap();
        let got = r.vector_at(0, dim).unwrap();
        for (a, b) in v.iter().zip(got) {
            assert_eq!(a.to_bits(), b.to_bits(), "f32 bits diverged");
        }
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn vector_truncated_never_panics() {
        let path = tmp_path("vector-truncated");
        let dim = 8;
        let raw: Vec<f32> = (0..dim * 500).map(|i| i as f32).collect();
        let vectors: Vec<Option<&[f32]>> = (0..500)
            .map(|i| Some(&raw[i * dim..(i + 1) * dim]))
            .collect();
        write_vector_segment(&path, 9, dim, &vectors).unwrap();
        let full = std::fs::metadata(&path).unwrap().len();
        let truncate_to = HEADER_LEN as u64 + 64;
        assert!(truncate_to < full);
        {
            let f = std::fs::OpenOptions::new().write(true).open(&path).unwrap();
            f.set_len(truncate_to).unwrap();
        }
        match SegmentReader::open(&path) {
            Ok(r) => {
                for id in 0..600u32 {
                    let _ = r.vector_at(id, dim); // must not panic
                }
                let _ = r.vectors_slice(dim);
            }
            Err(_) => { /* torn file rejected — also fine */ }
        }
        std::fs::remove_file(&path).ok();
    }

    // -----------------------------------------------------------------------
    // Variable-width column machinery (Phase 2e-A)
    // -----------------------------------------------------------------------

    /// Derive the inverted `terms` posting index from a dense `values` column —
    /// the same fold the seal does, so the keyword writer's two arguments stay
    /// consistent in the unit tests. Term `t` posts every docid whose value is `t`.
    fn kw_terms(
        values: &[Option<&str>],
    ) -> std::collections::BTreeMap<String, roaring::RoaringBitmap> {
        let mut terms: std::collections::BTreeMap<String, roaring::RoaringBitmap> =
            std::collections::BTreeMap::new();
        for (id, v) in values.iter().enumerate() {
            if let Some(s) = v {
                terms.entry((*s).to_string()).or_default().insert(id as u32);
            }
        }
        terms
    }

    /// Derive the inverted `elements` posting index from a dense `values` column
    /// — the same fold the Set seal does, so the set writer's two arguments stay
    /// consistent in the unit tests. Element `e` posts every docid whose member
    /// set contains `e` (Phase 2h-2).
    fn set_elems(
        values: &[Option<&[String]>],
    ) -> std::collections::BTreeMap<String, roaring::RoaringBitmap> {
        let mut elements: std::collections::BTreeMap<String, roaring::RoaringBitmap> =
            std::collections::BTreeMap::new();
        for (id, v) in values.iter().enumerate() {
            if let Some(members) = v {
                for m in members.iter() {
                    elements.entry(m.clone()).or_default().insert(id as u32);
                }
            }
        }
        elements
    }

    #[test]
    fn keyword_round_trip_small() {
        let path = tmp_path("keyword-rt-small");
        // Distinct dict (sorted): "apple"(0) "banana"(1) "cherry"(2).
        let values: Vec<Option<&str>> = vec![
            Some("banana"),
            None,
            Some("apple"),
            Some("cherry"),
            Some("banana"),
        ];
        write_keyword_segment(&path, 5, &values, &kw_terms(&values)).unwrap();

        let r = SegmentReader::open(&path).unwrap();
        assert_eq!(r.applied_seq(), 5);
        assert_eq!(r.n_docs(), 5);
        assert_eq!(r.keyword_at(0).as_deref(), Some("banana"));
        assert_eq!(r.keyword_at(1), None); // absent doc
        assert_eq!(r.keyword_at(2).as_deref(), Some("apple"));
        assert_eq!(r.keyword_at(3).as_deref(), Some("cherry"));
        assert_eq!(r.keyword_at(4).as_deref(), Some("banana"));
        assert_eq!(r.keyword_at(5), None); // id >= n_docs
        assert_eq!(r.keyword_at(u32::MAX), None);

        // INVERTED postings column (Phase 2h-1): each term's docid set + df read
        // straight off the segment, byte-identical to the in-RAM `terms` fold.
        assert_eq!(
            r.keyword_postings("banana"),
            Some([0u32, 4].into_iter().collect())
        );
        assert_eq!(
            r.keyword_postings("apple"),
            Some([2u32].into_iter().collect())
        );
        assert_eq!(
            r.keyword_postings("cherry"),
            Some([3u32].into_iter().collect())
        );
        assert_eq!(r.keyword_postings("durian"), None); // absent term
        assert_eq!(r.keyword_df("banana"), Some(2));
        assert_eq!(r.keyword_df("apple"), Some(1));
        assert_eq!(r.keyword_df("cherry"), Some(1));
        assert_eq!(r.keyword_df("durian"), None);
        std::fs::remove_file(&path).ok();
    }

    /// The docid-only posting-block codec round-trips exactly, incl an empty
    /// list, a single posting, a large-gap stream, and the u32::MAX edge.
    #[test]
    fn docid_block_codec_round_trip() {
        let cases: Vec<Vec<u32>> = vec![
            vec![],
            vec![0],
            vec![0, 1, 2, 3],
            vec![3, 7, 100, 100_000, 4_000_000_000],
            vec![u32::MAX],
        ];
        for docids in cases {
            let blob = encode_docid_block(&docids);
            // df == count prefix.
            let mut pos = 0usize;
            assert_eq!(read_varint(&blob, &mut pos).unwrap() as usize, docids.len());
            assert_eq!(
                decode_docid_block(&blob),
                Some(docids.clone()),
                "docids diverged"
            );
        }
        // A truncated blob must yield None, never panic.
        let blob = encode_docid_block(&[1, 2, 3]);
        assert!(decode_docid_block(&blob[..blob.len() - 1]).is_none());
    }

    /// The keyword posting column must span MULTIPLE 64KB LZ4 blocks and every
    /// term's posting list + df must still resolve through the skip-index.
    #[test]
    fn keyword_postings_multi_block() {
        let path = tmp_path("keyword-postings-multi-block");
        // ~12k distinct terms (one doc each) → several posting blocks, plus a few
        // hot terms shared by many docs to vary df.
        let n = 12_000usize;
        let owned: Vec<String> = (0..n).map(|i| format!("term-{i:08}")).collect();
        let mut values: Vec<Option<&str>> = owned.iter().map(|s| Some(s.as_str())).collect();
        // Append docs that all carry the very first term, so it has a fat df.
        let hot = owned[0].clone();
        for _ in 0..50 {
            values.push(Some(hot.as_str()));
        }
        let terms = kw_terms(&values);
        write_keyword_segment(&path, 1, &values, &terms).unwrap();

        let r = SegmentReader::open(&path).unwrap();
        // Spot-check terms across the dict-id space resolve to the right docs.
        for &i in &[0usize, 1, 1234, n / 2, n - 2, n - 1] {
            let want: roaring::RoaringBitmap =
                terms.get(owned[i].as_str()).unwrap().iter().collect();
            assert_eq!(
                r.keyword_postings(owned[i].as_str()),
                Some(want),
                "term {i}"
            );
            assert_eq!(
                r.keyword_df(owned[i].as_str()),
                Some(terms.get(owned[i].as_str()).unwrap().len()),
                "df {i}"
            );
        }
        // The hot term: docid 0 plus the 50 appended ids.
        assert_eq!(r.keyword_df(hot.as_str()), Some(51));
        std::fs::remove_file(&path).ok();
    }

    /// The dictionary must span MULTIPLE 64KB LZ4 blocks and every dict-id
    /// must still resolve through the skip-index binary-search + per-block
    /// prefix-delta reconstruction.
    #[test]
    fn keyword_multi_block_dict() {
        let path = tmp_path("keyword-multi-block");
        // ~20k distinct 30-byte keys with a long shared prefix (prefix-delta
        // makes a block dense) → forces several VAR_BLOCK_BYTES (64KB) blocks.
        let n = 20_000usize;
        let owned: Vec<String> = (0..n)
            .map(|i| format!("shared-prefix-key-{i:012}"))
            .collect();
        let values: Vec<Option<&str>> = owned.iter().map(|s| Some(s.as_str())).collect();
        write_keyword_segment(&path, 1, &values, &kw_terms(&values)).unwrap();

        let r = SegmentReader::open(&path).unwrap();
        assert_eq!(r.n_docs(), n as u32);
        // Probe across the whole id space (first, mid, last, and a stride) so a
        // skip-index block boundary cannot be skipped.
        for &id in &[0usize, 1, 1234, 9999, n / 2, n - 2, n - 1] {
            assert_eq!(
                r.keyword_at(id as u32).as_deref(),
                Some(owned[id].as_str()),
                "keyword id {id} diverged"
            );
        }
        for id in (0..n).step_by(517) {
            assert_eq!(r.keyword_at(id as u32).as_deref(), Some(owned[id].as_str()));
        }
        std::fs::remove_file(&path).ok();
    }

    /// The skip-index must locate the correct block for every dict-id even
    /// when the dictionary's block boundaries fall mid-run.
    #[test]
    fn dict_skip_index_locate() {
        let path = tmp_path("dict-locate");
        // Keys long enough that ~3000 of them span >1 block, with NO shared
        // prefix so suffixes are the full key (stresses the byte budget).
        let n = 3000usize;
        let owned: Vec<String> = (0..n)
            .map(|i| format!("{i:08}-{}", "x".repeat(40)))
            .collect();
        let values: Vec<Option<&str>> = owned.iter().map(|s| Some(s.as_str())).collect();
        write_keyword_segment(&path, 1, &values, &kw_terms(&values)).unwrap();
        let r = SegmentReader::open(&path).unwrap();
        // Every dict-id resolves to its own key (the dict is the sorted
        // distinct set, which == the input here since all keys are distinct).
        let mut sorted = owned.clone();
        sorted.sort();
        for (dict_id, want) in sorted.iter().enumerate() {
            let got = r.dict_string(dict_id as u32);
            assert_eq!(got.as_deref(), Some(want.as_str()), "dict id {dict_id}");
        }
        assert_eq!(r.dict_string(n as u32), None); // out of range
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn set_round_trip_small() {
        let path = tmp_path("set-rt-small");
        let d0: Vec<String> = vec!["red".into(), "green".into()];
        let d2: Vec<String> = vec!["blue".into()];
        let d3: Vec<String> = vec![]; // present-but-empty
        let values: Vec<Option<&[String]>> =
            vec![Some(&d0[..]), None, Some(&d2[..]), Some(&d3[..])];
        write_set_segment(&path, 7, &values, &set_elems(&values)).unwrap();

        let r = SegmentReader::open(&path).unwrap();
        assert_eq!(r.n_docs(), 4);
        // Members come back in the order they were stored.
        assert_eq!(
            r.set_at(0),
            Some(vec!["red".to_string(), "green".to_string()])
        );
        assert_eq!(r.set_at(1), None); // absent doc
        assert_eq!(r.set_at(2), Some(vec!["blue".to_string()]));
        assert_eq!(r.set_at(3), Some(vec![])); // present-but-empty
        assert_eq!(r.set_at(4), None); // id >= n_docs

        // INVERTED postings column (Phase 2h-2): each element's docid set + df
        // read straight off the segment, byte-identical to the in-RAM `elements`
        // fold. red→{0}, green→{0}, blue→{2}.
        assert_eq!(r.set_postings("red"), Some([0u32].into_iter().collect()));
        assert_eq!(r.set_postings("green"), Some([0u32].into_iter().collect()));
        assert_eq!(r.set_postings("blue"), Some([2u32].into_iter().collect()));
        assert_eq!(r.set_postings("absent"), None); // not in the dictionary
        assert_eq!(r.set_df("red"), Some(1));
        assert_eq!(r.set_df("blue"), Some(1));
        assert_eq!(r.set_df("absent"), None);
        // The full enumeration walks the dict in sorted order with postings.
        let all = r.set_elements_all().unwrap();
        assert_eq!(
            all,
            vec![
                ("blue".to_string(), [2u32].into_iter().collect()),
                ("green".to_string(), [0u32].into_iter().collect()),
                ("red".to_string(), [0u32].into_iter().collect()),
            ]
        );
        std::fs::remove_file(&path).ok();
    }

    /// CSR offsets round-trip: the packed member slice for each doc must be
    /// exactly `packed[offsets[i]..offsets[i+1]]`. Build a multi-valued corpus
    /// with varied cardinalities (incl 0) and assert the reconstructed member
    /// sets, before any dual-path wiring.
    #[test]
    fn set_csr_offsets_round_trip() {
        let path = tmp_path("set-csr");
        // doc i gets i % 4 members drawn from a shared pool.
        let pool = ["a", "b", "c", "d", "e", "f", "g"];
        let n = 200usize;
        let docs: Vec<Vec<String>> = (0..n)
            .map(|i| {
                (0..(i % 4))
                    .map(|j| pool[(i + j) % pool.len()].to_string())
                    .collect::<std::collections::BTreeSet<_>>() // dedupe + sort
                    .into_iter()
                    .collect()
            })
            .collect();
        let values: Vec<Option<&[String]>> = docs.iter().map(|d| Some(d.as_slice())).collect();
        let elems = set_elems(&values);
        write_set_segment(&path, 1, &values, &elems).unwrap();

        let r = SegmentReader::open(&path).unwrap();
        for (i, want) in docs.iter().enumerate() {
            let got = r.set_at(i as u32).unwrap();
            assert_eq!(
                &got, want,
                "doc {i} member slice diverged (CSR off-by-one?)"
            );
        }
        // Every element's INVERTED postings (Phase 2h-2) must equal the in-RAM
        // fold, and df its length — proves the parallel column round-trips
        // through the skip-index over a multi-valued corpus.
        for (el, want) in &elems {
            assert_eq!(
                r.set_postings(el).as_ref(),
                Some(want),
                "set element `{el}` postings diverged"
            );
            assert_eq!(
                r.set_df(el),
                Some(want.len()),
                "set element `{el}` df diverged"
            );
        }
        std::fs::remove_file(&path).ok();
    }

    /// A truncated VAR block (the dict region is chopped so a dict-id's frame
    /// overruns the file) must return `None`, never panic. The fixed columns
    /// (dict-id / present) survive, so `keyword_at` reaches the dict read and
    /// the torn-frame guard fires.
    #[test]
    fn truncated_var_block_returns_none() {
        let path = tmp_path("var-truncated");
        let n = 4000usize;
        let owned: Vec<String> = (0..n).map(|i| format!("key-{i:08}")).collect();
        let values: Vec<Option<&str>> = owned.iter().map(|s| Some(s.as_str())).collect();
        write_keyword_segment(&path, 1, &values, &kw_terms(&values)).unwrap();

        // Forge a file whose directory + footer are intact (crc still matches)
        // but the VAR region is chopped so the dict column overruns the file.
        // Same technique as `truncated_mid_column_with_intact_footer_returns_none`.
        let original = std::fs::read(&path).unwrap();
        let len = original.len();
        let footer = Footer::from_bytes(&original[len - FOOTER_LEN..len]).unwrap();
        let dir_off = footer.dir_offset as usize;
        let dir_end = dir_off + footer.dir_len as usize;

        // Find the dict column's byte_offset by decoding the directory.
        let dir: Vec<ColumnRef> = ciborium::from_reader(&original[dir_off..dir_end]).unwrap();
        let dict = dir.iter().find(|c| c.role == ROLE_DICT).unwrap();
        let dict_off = dict.byte_offset as usize;
        // Keep everything up to a few bytes into the dict region, then re-append
        // the unchanged directory + footer (rewriting only dir_offset).
        let keep = dict_off + 8; // a few bytes into the first frame
        assert!(keep < dir_off);
        let mut forged = Vec::new();
        forged.extend_from_slice(&original[..keep]);
        let new_dir_off = forged.len() as u64;
        forged.extend_from_slice(&original[dir_off..dir_end]); // same crc
        let new_footer = Footer {
            dir_offset: new_dir_off,
            dir_len: footer.dir_len,
            crc32: footer.crc32,
            magic2: footer.magic2,
        };
        forged.extend_from_slice(&new_footer.to_bytes());
        std::fs::write(&path, &forged).unwrap();

        match SegmentReader::open(&path) {
            Ok(r) => {
                // The dict-id column may survive; the dict block read overruns
                // the chopped file → None for every id, never a panic.
                for id in 0..n as u32 + 10 {
                    let _ = r.keyword_at(id);
                }
                // At least the high ids (whose dict block is past `keep`) miss.
                assert_eq!(r.keyword_at(n as u32 - 1), None);
            }
            Err(_) => { /* also acceptable */ }
        }
        std::fs::remove_file(&path).ok();
    }

    /// The decompressed-block moka cache must serve a repeated dict read from
    /// the cache (cold miss → warm hit), returning the identical string.
    #[test]
    fn var_block_cache_hit() {
        let path = tmp_path("var-cache");
        let owned: Vec<String> = (0..500).map(|i| format!("k{i:05}")).collect();
        let values: Vec<Option<&str>> = owned.iter().map(|s| Some(s.as_str())).collect();
        write_keyword_segment(&path, 1, &values, &kw_terms(&values)).unwrap();
        let r = SegmentReader::open(&path).unwrap();
        let first = r.keyword_at(123);
        let again = r.keyword_at(123); // served from the cached block
        assert_eq!(first, again);
        assert_eq!(first.as_deref(), Some("k00123"));
        std::fs::remove_file(&path).ok();
    }

    // -----------------------------------------------------------------------
    // Text segment (Phase 2e-B)
    // -----------------------------------------------------------------------

    /// The varint + delta posting-block codec must round-trip exactly, incl an
    /// empty list, a single posting, and a large-gap / high-tf stream.
    #[test]
    fn posting_block_codec_round_trip() {
        let cases: Vec<(Vec<u32>, Vec<u32>)> = vec![
            (vec![], vec![]),
            (vec![0], vec![1]),
            (vec![0, 1, 2, 3], vec![5, 4, 3, 2]),
            (
                vec![3, 7, 100, 100_000, 4_000_000_000],
                vec![1, 2, 3, 4, 999],
            ),
            (vec![u32::MAX], vec![u32::MAX]),
        ];
        for (docids, tfs) in cases {
            let blob = encode_posting_block(&docids, &tfs);
            let (d2, t2) = decode_posting_block(&blob).expect("decode");
            assert_eq!(d2, docids, "docids diverged");
            assert_eq!(t2, tfs, "tfs diverged");
        }
        // A truncated blob must yield None, never panic.
        let blob = encode_posting_block(&[1, 2, 3], &[1, 1, 1]);
        assert!(decode_posting_block(&blob[..blob.len() - 1]).is_none());
    }

    /// `write_text_segment` round-trip: a small text corpus's stored postings,
    /// doc-len column, and header scalars all read back exactly.
    #[test]
    fn text_segment_round_trip() {
        use crate::storage::Postings;
        let path = tmp_path("text-rt");
        // Tokens (dict order is ascending — BTreeMap): "apple", "banana", "cherry".
        let mut tokens: std::collections::BTreeMap<String, Postings> =
            std::collections::BTreeMap::new();
        tokens.insert(
            "apple".into(),
            Postings::from_sorted(vec![0, 2, 3], vec![3, 1, 2]),
        );
        tokens.insert(
            "banana".into(),
            Postings::from_sorted(vec![1, 3], vec![5, 1]),
        );
        tokens.insert("cherry".into(), Postings::from_sorted(vec![0], vec![1]));
        let lens: Vec<u32> = vec![4, 5, 1, 3]; // doc lengths; doc with len 0 = absent
        let doc_count: u64 = 4;
        let total_doc_len: u64 = 4 + 5 + 1 + 3;

        write_text_segment(&path, 42, &tokens, &lens, doc_count, total_doc_len).unwrap();
        let r = SegmentReader::open(&path).unwrap();

        assert_eq!(r.applied_seq(), 42);
        assert_eq!(r.n_docs(), 4);
        assert_eq!(r.text_doc_count(), doc_count);
        assert_eq!(r.text_total_doc_len(), total_doc_len);

        // Postings round-trip exactly, per token.
        assert_eq!(
            r.text_postings("apple"),
            Some((vec![0, 2, 3], vec![3, 1, 2]))
        );
        assert_eq!(r.text_postings("banana"), Some((vec![1, 3], vec![5, 1])));
        assert_eq!(r.text_postings("cherry"), Some((vec![0], vec![1])));
        assert_eq!(r.text_postings("durian"), None); // absent token

        // df == stored posting length.
        assert_eq!(r.text_token_df("apple"), 3);
        assert_eq!(r.text_token_df("banana"), 2);
        assert_eq!(r.text_token_df("cherry"), 1);
        assert_eq!(r.text_token_df("durian"), 0);

        // DocLen column read zero-copy.
        for (id, &l) in lens.iter().enumerate() {
            assert_eq!(r.text_doc_len(id as u32), l, "doclen id {id}");
        }
        assert_eq!(r.text_doc_len(99), 0); // out of range -> 0

        std::fs::remove_file(&path).ok();
    }

    // -----------------------------------------------------------------------
    // Collection EID column (Phase 2f-1)
    // -----------------------------------------------------------------------

    /// The eid-by-position column round-trips: `eid_at(i)` returns the i-th
    /// external_id and `eids_all` reproduces the whole dense Vec in order, even
    /// across awkward strings (empty / unicode / long).
    #[test]
    fn eid_segment_round_trip() {
        let path = tmp_path("eid-rt");
        let owned: Vec<String> = vec![
            "doc-0".into(),
            "".into(),           // empty eid is legal
            "日本語-doc".into(), // non-ascii
            "x".repeat(300),     // long, crosses no prefix-share
            "doc-0".into(),      // duplicate STRING but distinct docid (by position)
        ];
        let eids: Vec<&str> = owned.iter().map(|s| s.as_str()).collect();
        write_eid_segment(&path, 99, &eids).unwrap();

        let r = SegmentReader::open(&path).unwrap();
        assert_eq!(r.applied_seq(), 99);
        assert_eq!(r.eid_count(), owned.len() as u32);
        for (i, want) in owned.iter().enumerate() {
            assert_eq!(
                r.eid_at(i as u32).as_deref(),
                Some(want.as_str()),
                "eid {i}"
            );
        }
        assert_eq!(r.eid_at(owned.len() as u32), None); // out of range
        assert_eq!(r.eids_all().as_deref(), Some(&owned[..]));
        std::fs::remove_file(&path).ok();
    }

    /// The eid column must survive crossing multiple 64KB LZ4 blocks and resolve
    /// every position through the skip-index.
    #[test]
    fn eid_segment_multi_block() {
        let path = tmp_path("eid-multi-block");
        let n = 30_000usize;
        let owned: Vec<String> = (0..n).map(|i| format!("external-id-{i:012}")).collect();
        let eids: Vec<&str> = owned.iter().map(|s| s.as_str()).collect();
        write_eid_segment(&path, 1, &eids).unwrap();
        let r = SegmentReader::open(&path).unwrap();
        assert_eq!(r.eid_count(), n as u32);
        for &i in &[0usize, 1, 1234, n / 2, n - 2, n - 1] {
            assert_eq!(
                r.eid_at(i as u32).as_deref(),
                Some(owned[i].as_str()),
                "eid {i}"
            );
        }
        let all = r.eids_all().unwrap();
        assert_eq!(all.len(), n);
        assert_eq!(all[n - 1], owned[n - 1]);
        std::fs::remove_file(&path).ok();
    }

    /// The token dictionary must span MULTIPLE 64KB LZ4 blocks and every token's
    /// posting block must still resolve through the skip-index binary-search.
    #[test]
    fn text_segment_multi_block() {
        use crate::storage::Postings;
        let path = tmp_path("text-multi-block");
        let n_tokens = 20_000usize;
        let n_docs = 200u32;
        let mut tokens: std::collections::BTreeMap<String, Postings> =
            std::collections::BTreeMap::new();
        for i in 0..n_tokens {
            // Each token posts to a couple of docs with deterministic tf.
            let d0 = (i as u32) % n_docs;
            let d1 = ((i as u32) * 7 + 3) % n_docs;
            let (docids, tfs) = if d0 == d1 {
                (vec![d0], vec![(i % 7 + 1) as u32])
            } else if d0 < d1 {
                (vec![d0, d1], vec![(i % 7 + 1) as u32, (i % 3 + 1) as u32])
            } else {
                (vec![d1, d0], vec![(i % 3 + 1) as u32, (i % 7 + 1) as u32])
            };
            tokens.insert(
                format!("shared-prefix-token-{i:012}"),
                Postings::from_sorted(docids, tfs),
            );
        }
        let lens: Vec<u32> = (0..n_docs).map(|d| (d % 9) + 1).collect();
        write_text_segment(
            &path,
            1,
            &tokens,
            &lens,
            n_docs as u64,
            lens.iter().map(|&l| l as u64).sum(),
        )
        .unwrap();
        let r = SegmentReader::open(&path).unwrap();
        assert_eq!(r.n_docs(), n_docs);
        // Probe across the whole token space so no skip-index block is missed.
        for &i in &[
            0usize,
            1,
            1234,
            9999,
            n_tokens / 2,
            n_tokens - 2,
            n_tokens - 1,
        ] {
            let tok = format!("shared-prefix-token-{i:012}");
            let want = tokens.get(&tok).unwrap();
            assert_eq!(
                r.text_postings(&tok),
                Some((want.docids().to_vec(), want.tfs().to_vec())),
                "token {i} postings diverged"
            );
            assert_eq!(r.text_token_df(&tok), want.docids().len());
        }
        std::fs::remove_file(&path).ok();
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/lumen/src/segment.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `projects/lumen/src/segment.rs` captured during lumen
      standardization onto the per-file codegen ladder.
```

// pep658_fallback.rs — orchestrator for the "no .metadata sidecar"
// branch of metadata discovery.
//
// PEP 658 lets a Simple-Repository index advertise a separate
// `.metadata` sidecar alongside every wheel URL (handled by
// pep658_url.rs, Tick 101). When the index does NOT advertise one,
// we fall back to pulling METADATA out of the wheel itself without
// downloading the full archive — the classic uv/pip trick that
// composes:
//
//     Range (Tick 94)              ← fetch the tail
//   → parse_zip_tail (T102)        ← find the Central Directory
//   → Range                        ← fetch the CD bytes
//   → walk_central_directory (T103) ← locate the METADATA entry
//   → Range                        ← fetch the LFH + payload
//   → parse_lfh (T104)             ← decode header_size
//   → inflate_metadata (T105)      ← decompress METADATA bytes
//   → verify_crc32 (T106)          ← integrity check
//
// This module ties them together behind a single typed entry-point
// so the resolver can call `fetch_metadata_via_range(url, fetcher,
// total_size)` and get a `Vec<u8>` back.
//
// Network I/O is injected via the `RangeFetcher` trait, so the
// orchestrator stays synchronous + free of `reqwest`/`tokio`. The
// caller's HTTP layer (or a mock in tests) implements one method.
//
// Two-shot fetch pattern (matching pip + uv):
//
//   1. First Range: last 64 KiB to cover the EOCD and (usually) the
//      whole Central Directory of a small wheel. If parse_zip_tail
//      reports the CD extends earlier than the tail covers, issue a
//      second Range for the missing CD bytes.
//   2. Second Range: LFH + compressed payload for METADATA, sized
//      from the CDR entry. We pad by 256 bytes for the variable
//      LFH (extras can grow), then read again if parse_lfh needs
//      more.
//
// Out of scope:
//   * Connection reuse / HTTP keepalive — RangeFetcher is the caller's
//     responsibility.
//   * Caching the CD bytes for sibling wheels in the same archive
//     (PKZIP-level batch fetch) — wheels are independent.

use crate::pkgmanage::pkgmgr::crc32_verify::verify_against_lfh;
use crate::pkgmanage::pkgmgr::deflate_inflate::inflate_metadata;
use crate::pkgmanage::pkgmgr::range::ByteRange;
use crate::pkgmanage::pkgmgr::types::IndexError;
use crate::pkgmanage::pkgmgr::zip_cdr::{find_metadata_entry, walk_central_directory, CdEntry};
use crate::pkgmanage::pkgmgr::zip_lfh::parse_lfh;
use crate::pkgmanage::pkgmgr::zip_tail::{parse_zip_tail, EocdInfo};

/// How many bytes the initial tail fetch should cover. Sized to
/// capture both the EOCD (≤ 22 bytes + 64 KiB comment) and the
/// Central Directory of a typical wheel (~40 entries × 100 bytes).
/// Larger than uv's default (4 KiB) because we'd rather pay one
/// round-trip than two on the median wheel.
pub const DEFAULT_TAIL_SIZE: u64 = 64 * 1024;

/// Padding added to the LFH range to cover the variable
/// filename/extra region. Wheels rarely emit extras, so 256 bytes
/// is sufficient for filename ≤ ~200 chars + a Zip64 extra block.
const LFH_VARIABLE_PADDING: u64 = 256;

/// Pluggable HTTP byte-range fetcher. Implementations should return
/// exactly the bytes the server delivered for the request range;
/// the orchestrator handles short responses + cross-checks lengths.
pub trait RangeFetcher {
    /// Fetch the bytes implied by `range` relative to `url`. The
    /// caller is the source of truth for the archive's total size,
    /// so this trait does not return it.
    fn fetch_range(&self, url: &str, range: &ByteRange) -> Result<Vec<u8>, IndexError>;
}

/// Driver for the fallback flow. Composes one full PEP 658 fallback
/// fetch into a single typed call.
///
/// `total_size` is the archive size (from a prior HEAD or
/// Content-Length probe). It's required because Suffix-range fetches
/// need to know the archive offset of the tail window for Zip64
/// validation in [`parse_zip_tail`].
pub fn fetch_metadata_via_range<F: RangeFetcher>(
    url: &str,
    fetcher: &F,
    total_size: u64,
) -> Result<Vec<u8>, IndexError> {
    if total_size == 0 {
        return Err(IndexError::ParseError {
            url: url.to_string(),
            detail: "archive total_size of 0 cannot contain a wheel".into(),
        });
    }

    // ---- step 1: tail fetch -----------------------------------------
    let tail_size = DEFAULT_TAIL_SIZE.min(total_size);
    let tail = fetcher.fetch_range(url, &ByteRange::Suffix { len: tail_size })?;
    if tail.is_empty() {
        return Err(IndexError::ParseError {
            url: url.to_string(),
            detail: "empty response to tail Range request".into(),
        });
    }
    let eocd: EocdInfo = parse_zip_tail(&tail)?;

    // ---- step 2: locate CD ------------------------------------------
    let tail_archive_start = total_size.saturating_sub(tail.len() as u64);
    let cd_bytes = read_central_directory(url, fetcher, &eocd, &tail, tail_archive_start)?;
    let entries = walk_central_directory(&cd_bytes)?;
    let meta = find_metadata_entry(&entries).ok_or_else(|| IndexError::ParseError {
        url: url.to_string(),
        detail: "wheel Central Directory has no `.dist-info/METADATA` entry".into(),
    })?;

    // ---- step 3: LFH + payload --------------------------------------
    let payload = read_payload(url, fetcher, meta, total_size)?;

    // ---- step 4: parse LFH for header_size + CRC --------------------
    let lfh = parse_lfh(&payload)?;
    if lfh.is_encrypted() {
        return Err(IndexError::ParseError {
            url: url.to_string(),
            detail: "wheel METADATA entry is marked encrypted; refusing to inflate".into(),
        });
    }
    let header_size = lfh.header_size as usize;
    let compressed_size = meta.compressed_size as usize;
    let end = header_size
        .checked_add(compressed_size)
        .ok_or_else(|| IndexError::ParseError {
            url: url.to_string(),
            detail: "LFH header_size + compressed_size overflowed usize".into(),
        })?;
    if end > payload.len() {
        return Err(IndexError::ParseError {
            url: url.to_string(),
            detail: format!(
                "fetched payload too short: needed {end} bytes (header {header_size} + \
                 compressed {compressed_size}), got {}",
                payload.len()
            ),
        });
    }
    let compressed = &payload[header_size..end];

    // ---- step 5: inflate --------------------------------------------
    let bytes = inflate_metadata(
        meta.compression_method,
        compressed,
        Some(meta.uncompressed_size),
    )?;

    // ---- step 6: CRC verify -----------------------------------------
    verify_against_lfh(&bytes, &lfh)?;

    Ok(bytes)
}

/// Fetch the Central Directory bytes. If the initial tail already
/// contained the CD (true for small wheels with short comments),
/// slice it out without a second round-trip.
fn read_central_directory<F: RangeFetcher>(
    url: &str,
    fetcher: &F,
    eocd: &EocdInfo,
    tail: &[u8],
    tail_archive_start: u64,
) -> Result<Vec<u8>, IndexError> {
    let cd_size_usize = usize::try_from(eocd.cd_size).map_err(|_| IndexError::ParseError {
        url: url.to_string(),
        detail: "Central Directory size exceeds addressable memory".into(),
    })?;

    if eocd.cd_offset >= tail_archive_start {
        let rel = (eocd.cd_offset - tail_archive_start) as usize;
        let end = rel.checked_add(cd_size_usize).ok_or_else(|| IndexError::ParseError {
            url: url.to_string(),
            detail: "Central Directory slice overflow".into(),
        })?;
        if end <= tail.len() {
            return Ok(tail[rel..end].to_vec());
        }
    }

    // Outside the tail window — issue a second Range request.
    let cd_end = eocd
        .cd_offset
        .checked_add(eocd.cd_size)
        .ok_or_else(|| IndexError::ParseError {
            url: url.to_string(),
            detail: "Central Directory absolute end overflowed u64".into(),
        })?;
    let bytes = fetcher.fetch_range(
        url,
        &ByteRange::Bounded {
            start: eocd.cd_offset,
            end: cd_end - 1, // inclusive
        },
    )?;
    if bytes.len() != cd_size_usize {
        return Err(IndexError::ParseError {
            url: url.to_string(),
            detail: format!(
                "CD Range response length mismatch: expected {} bytes, got {}",
                cd_size_usize,
                bytes.len()
            ),
        });
    }
    Ok(bytes)
}

/// Fetch the LFH + compressed payload for the METADATA entry. We
/// pad the request by [`LFH_VARIABLE_PADDING`] to cover the
/// variable-length filename + extra region.
fn read_payload<F: RangeFetcher>(
    url: &str,
    fetcher: &F,
    meta: &CdEntry,
    total_size: u64,
) -> Result<Vec<u8>, IndexError> {
    let start = meta.local_header_offset;
    // 30-byte fixed LFH + filename/extra padding + compressed bytes.
    let length = 30u64
        .checked_add(LFH_VARIABLE_PADDING)
        .and_then(|n| n.checked_add(meta.compressed_size))
        .ok_or_else(|| IndexError::ParseError {
            url: url.to_string(),
            detail: "LFH + payload range overflow".into(),
        })?;
    let end_exclusive = start.checked_add(length).ok_or_else(|| IndexError::ParseError {
        url: url.to_string(),
        detail: "LFH range absolute end overflowed u64".into(),
    })?;
    let end_clamped = end_exclusive.min(total_size);
    if end_clamped <= start {
        return Err(IndexError::ParseError {
            url: url.to_string(),
            detail: format!(
                "LFH range collapsed: start={start}, end={end_clamped}, total={total_size}"
            ),
        });
    }
    fetcher.fetch_range(
        url,
        &ByteRange::Bounded {
            start,
            end: end_clamped - 1,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pkgmanage::pkgmgr::deflate_inflate::METHOD_DEFLATE;
    use flate2::write::DeflateEncoder;
    use flate2::Compression;
    use std::cell::RefCell;
    use std::io::Write;

    fn err_detail(err: IndexError) -> String {
        match err {
            IndexError::ParseError { detail, .. } => detail,
            other => panic!("expected ParseError, got {other:?}"),
        }
    }

    /// In-memory wheel builder: writes one LFH + payload entry, one
    /// CDR record, and one EOCD. Only the METADATA entry is needed.
    fn build_wheel(metadata: &[u8]) -> Vec<u8> {
        // Compress METADATA with raw deflate.
        let mut enc = DeflateEncoder::new(Vec::new(), Compression::default());
        enc.write_all(metadata).unwrap();
        let compressed = enc.finish().unwrap();
        let crc = crc32fast::hash(metadata);

        let filename = b"pkg-1.0.dist-info/METADATA";

        let mut archive = Vec::new();

        // ---- LFH ---------------------------------------------------
        let lfh_offset = archive.len() as u32;
        archive.extend_from_slice(&[0x50, 0x4b, 0x03, 0x04]); // sig
        archive.extend_from_slice(&20u16.to_le_bytes()); // version needed
        archive.extend_from_slice(&0u16.to_le_bytes()); // gp flag
        archive.extend_from_slice(&8u16.to_le_bytes()); // method = deflate
        archive.extend_from_slice(&0u16.to_le_bytes()); // mod time
        archive.extend_from_slice(&0u16.to_le_bytes()); // mod date
        archive.extend_from_slice(&crc.to_le_bytes());
        archive.extend_from_slice(&(compressed.len() as u32).to_le_bytes());
        archive.extend_from_slice(&(metadata.len() as u32).to_le_bytes());
        archive.extend_from_slice(&(filename.len() as u16).to_le_bytes());
        archive.extend_from_slice(&0u16.to_le_bytes()); // extra len = 0
        archive.extend_from_slice(filename);
        archive.extend_from_slice(&compressed);

        // ---- CDR ---------------------------------------------------
        let cd_offset = archive.len() as u32;
        archive.extend_from_slice(&[0x50, 0x4b, 0x01, 0x02]); // sig
        archive.extend_from_slice(&20u16.to_le_bytes()); // version made
        archive.extend_from_slice(&20u16.to_le_bytes()); // version needed
        archive.extend_from_slice(&0u16.to_le_bytes()); // gp flag
        archive.extend_from_slice(&8u16.to_le_bytes()); // method
        archive.extend_from_slice(&0u16.to_le_bytes()); // mod time
        archive.extend_from_slice(&0u16.to_le_bytes()); // mod date
        archive.extend_from_slice(&crc.to_le_bytes());
        archive.extend_from_slice(&(compressed.len() as u32).to_le_bytes());
        archive.extend_from_slice(&(metadata.len() as u32).to_le_bytes());
        archive.extend_from_slice(&(filename.len() as u16).to_le_bytes());
        archive.extend_from_slice(&0u16.to_le_bytes()); // extra
        archive.extend_from_slice(&0u16.to_le_bytes()); // comment
        archive.extend_from_slice(&0u16.to_le_bytes()); // disk num
        archive.extend_from_slice(&0u16.to_le_bytes()); // internal attrs
        archive.extend_from_slice(&0u32.to_le_bytes()); // external attrs
        archive.extend_from_slice(&lfh_offset.to_le_bytes());
        archive.extend_from_slice(filename);

        let cd_size = (archive.len() as u32) - cd_offset;

        // ---- EOCD --------------------------------------------------
        archive.extend_from_slice(&[0x50, 0x4b, 0x05, 0x06]); // sig
        archive.extend_from_slice(&0u16.to_le_bytes()); // disk number
        archive.extend_from_slice(&0u16.to_le_bytes()); // disk w/ CD
        archive.extend_from_slice(&1u16.to_le_bytes()); // entries on disk
        archive.extend_from_slice(&1u16.to_le_bytes()); // total entries
        archive.extend_from_slice(&cd_size.to_le_bytes());
        archive.extend_from_slice(&cd_offset.to_le_bytes());
        archive.extend_from_slice(&0u16.to_le_bytes()); // comment len = 0

        archive
    }

    struct FakeFetcher {
        archive: Vec<u8>,
        call_count: RefCell<usize>,
    }

    impl FakeFetcher {
        fn new(archive: Vec<u8>) -> Self {
            Self {
                archive,
                call_count: RefCell::new(0),
            }
        }
        fn call_count(&self) -> usize {
            *self.call_count.borrow()
        }
    }

    impl RangeFetcher for FakeFetcher {
        fn fetch_range(&self, _url: &str, range: &ByteRange) -> Result<Vec<u8>, IndexError> {
            *self.call_count.borrow_mut() += 1;
            let total = self.archive.len() as u64;
            let (start, end_excl) = match range {
                ByteRange::Bounded { start, end } => (*start, end + 1),
                ByteRange::From { start } => (*start, total),
                ByteRange::Suffix { len } => (total.saturating_sub(*len), total),
            };
            if start > total {
                return Ok(Vec::new());
            }
            let end_clamped = end_excl.min(total);
            Ok(self.archive[start as usize..end_clamped as usize].to_vec())
        }
    }

    #[test]
    fn happy_path_roundtrip() {
        let metadata = b"Metadata-Version: 2.1\nName: pkg\nVersion: 1.0\n";
        let archive = build_wheel(metadata);
        let total = archive.len() as u64;
        let fetcher = FakeFetcher::new(archive);

        let out = fetch_metadata_via_range("https://x.example/pkg-1.0-py3-none-any.whl", &fetcher, total)
            .unwrap();
        assert_eq!(out, metadata);
        // For a small wheel everything fits in the tail → exactly 2
        // fetches: tail, then LFH+payload (CD slice was inside tail).
        assert_eq!(fetcher.call_count(), 2);
    }

    #[test]
    fn metadata_at_realistic_size_two_fetches() {
        let metadata = "\
Metadata-Version: 2.1
Name: requests
Version: 2.31.0
Summary: Python HTTP for Humans.
Requires-Dist: charset-normalizer (>=2,<4)
Requires-Dist: idna (>=2.5,<4)
Requires-Dist: urllib3 (>=1.21.1,<3)
Requires-Dist: certifi (>=2017.4.17)
"
        .repeat(20); // ~5 KiB METADATA
        let archive = build_wheel(metadata.as_bytes());
        let total = archive.len() as u64;
        let fetcher = FakeFetcher::new(archive);

        let out = fetch_metadata_via_range("https://x.example/x.whl", &fetcher, total).unwrap();
        assert_eq!(out, metadata.as_bytes());
        // Still 2 fetches — wheel < DEFAULT_TAIL_SIZE.
        assert_eq!(fetcher.call_count(), 2);
    }

    #[test]
    fn large_incompressible_wheel_round_trips() {
        // Archive far larger than DEFAULT_TAIL_SIZE → exercises the
        // multi-Range code path even when the CD ends up inside the
        // tail (PKZIP places the CD right before EOCD, so single-
        // entry wheels almost always have an in-tail CD).
        //
        // Drive a xorshift32 PRNG to get genuinely incompressible
        // bytes so the resulting archive size is predictable.
        let mut state: u32 = 0x12345678;
        let mut big_metadata = Vec::with_capacity(300_000);
        for _ in 0..300_000 {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            big_metadata.push(state as u8);
        }
        let archive = build_wheel(&big_metadata);
        let total = archive.len() as u64;
        assert!(total > 2 * DEFAULT_TAIL_SIZE);
        let fetcher = FakeFetcher::new(archive);

        let out = fetch_metadata_via_range("https://x.example/big.whl", &fetcher, total).unwrap();
        assert_eq!(out, big_metadata);
    }

    #[test]
    fn cd_outside_tail_triggers_extra_fetch() {
        // Stuff a giant EOCD comment after the EOCD so the 64 KiB
        // tail window doesn't reach back to the Central Directory.
        // This is the exact shape that forces read_central_directory
        // into the "issue a second Range" branch.
        let metadata = b"Metadata-Version: 2.1\nName: x\n";
        let mut archive = build_wheel(metadata);
        // Locate the EOCD comment-length field (last 2 bytes of EOCD)
        // and overwrite it with 60 000, then append that many zero
        // bytes as the comment payload.
        // Pick a comment length C such that:
        //   * EOCD signature stays inside the 64 KiB tail
        //     → 22 + C ≤ DEFAULT_TAIL_SIZE
        //   * CD (≈76 bytes) sits OUTSIDE the tail
        //     → 22 + C + cd_size > DEFAULT_TAIL_SIZE
        // 65 500 satisfies both (signature at tail offset 14;
        // CD ends 22 bytes earlier than that, i.e. just outside).
        let comment_len: u16 = 65_500;
        let archive_len = archive.len();
        archive[archive_len - 2..].copy_from_slice(&comment_len.to_le_bytes());
        archive.extend(vec![0u8; comment_len as usize]);

        let total = archive.len() as u64;
        let fetcher = FakeFetcher::new(archive);
        let out = fetch_metadata_via_range("u", &fetcher, total).unwrap();
        assert_eq!(out, metadata);
        // Tail (Suffix) → CD (Bounded) → LFH+payload (Bounded) = 3.
        assert_eq!(fetcher.call_count(), 3);
    }

    #[test]
    fn rejects_zero_total_size() {
        let fetcher = FakeFetcher::new(vec![]);
        let err = fetch_metadata_via_range("u", &fetcher, 0).unwrap_err();
        assert!(err_detail(err).contains("cannot contain a wheel"));
    }

    #[test]
    fn rejects_empty_tail_response() {
        struct Empty;
        impl RangeFetcher for Empty {
            fn fetch_range(&self, _url: &str, _range: &ByteRange) -> Result<Vec<u8>, IndexError> {
                Ok(Vec::new())
            }
        }
        let err = fetch_metadata_via_range("u", &Empty, 1000).unwrap_err();
        assert!(err_detail(err).contains("empty response to tail"));
    }

    #[test]
    fn detects_missing_metadata_entry() {
        // Build a wheel but rewrite the CDR filename to something
        // that doesn't end in .dist-info/METADATA.
        let archive = build_wheel(b"x");
        let mut tampered = archive.clone();
        // Find the CDR filename region — exhaustive scan for
        // "dist-info/METADATA" and overwrite first match.
        let needle = b"dist-info/METADATA";
        let mut occurrences = Vec::new();
        for i in 0..tampered.len().saturating_sub(needle.len()) {
            if &tampered[i..i + needle.len()] == needle {
                occurrences.push(i);
            }
        }
        // Both LFH and CDR contain the filename. Rewrite *both* so
        // walk_central_directory + find_metadata_entry come up empty.
        for off in occurrences {
            // Overwrite "M" of METADATA with "X" — keeps length.
            tampered[off + 10] = b'X';
        }

        let total = tampered.len() as u64;
        let fetcher = FakeFetcher::new(tampered);
        let err = fetch_metadata_via_range("u", &fetcher, total).unwrap_err();
        assert!(err_detail(err).contains("no `.dist-info/METADATA`"));
    }

    #[test]
    fn crc_mismatch_detected() {
        // Tamper with the compressed payload bytes after CRC was set.
        let metadata = b"original";
        let mut archive = build_wheel(metadata);
        // Find the compressed payload region (between LFH end and
        // CDR start) and flip a byte in it.
        // LFH = 30 fixed + 26 filename = 56 bytes; payload starts at 56.
        archive[60] ^= 0xFF;

        let total = archive.len() as u64;
        let fetcher = FakeFetcher::new(archive);
        let err = fetch_metadata_via_range("u", &fetcher, total).unwrap_err();
        let detail = err_detail(err);
        // Could surface as deflate decode error OR CRC mismatch
        // depending on which byte we hit. Both are acceptable
        // detection outcomes — we just want the orchestrator to fail.
        assert!(
            detail.contains("CRC")
                || detail.contains("deflate")
                || detail.contains("mismatch")
                || detail.contains("error"),
            "unexpected error detail: {detail}"
        );
    }

    #[test]
    fn rejects_encrypted_metadata_entry() {
        // Manually flip bit 0 (encryption) in the LFH gp-flag, so
        // parse_lfh reports `is_encrypted` and the orchestrator
        // rejects before inflating garbage.
        let mut archive = build_wheel(b"Metadata-Version: 2.1\n");
        // LFH gp_flag is at bytes [6..8] of the LFH (offset 0 in our
        // wheel because the LFH starts at byte 0).
        archive[6] = 0x01;

        let total = archive.len() as u64;
        let fetcher = FakeFetcher::new(archive);
        let err = fetch_metadata_via_range("u", &fetcher, total).unwrap_err();
        assert!(err_detail(err).contains("encrypted"));
    }

    #[test]
    fn deflate_method_constant_round_trip() {
        // Sanity: orchestrator dispatches on METHOD_DEFLATE for
        // wheels (matches build_wheel which always writes 8).
        assert_eq!(METHOD_DEFLATE, 8);
    }

    #[test]
    fn fetches_use_correct_ranges() {
        // Sanity: the fetcher sees a Suffix first, then a Bounded
        // for the LFH+payload (CD was in-tail for this small wheel).
        struct RecordingFetcher {
            archive: Vec<u8>,
            ranges: RefCell<Vec<ByteRange>>,
        }
        impl RangeFetcher for RecordingFetcher {
            fn fetch_range(&self, _url: &str, range: &ByteRange) -> Result<Vec<u8>, IndexError> {
                self.ranges.borrow_mut().push(range.clone());
                let total = self.archive.len() as u64;
                let (start, end_excl) = match range {
                    ByteRange::Bounded { start, end } => (*start, end + 1),
                    ByteRange::From { start } => (*start, total),
                    ByteRange::Suffix { len } => (total.saturating_sub(*len), total),
                };
                Ok(self.archive[start as usize..end_excl.min(total) as usize].to_vec())
            }
        }

        let metadata = b"Metadata-Version: 2.1\nName: x\n";
        let archive = build_wheel(metadata);
        let total = archive.len() as u64;
        let f = RecordingFetcher {
            archive,
            ranges: RefCell::new(Vec::new()),
        };
        let _ = fetch_metadata_via_range("u", &f, total).unwrap();
        let ranges = f.ranges.borrow();
        assert_eq!(ranges.len(), 2);
        assert!(matches!(ranges[0], ByteRange::Suffix { .. }));
        assert!(matches!(ranges[1], ByteRange::Bounded { .. }));
    }
}

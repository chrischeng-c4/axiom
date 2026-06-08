// deflate_inflate.rs — typed inflate wrapper for the PEP 658 fallback.
//
// Closes the chain we started in Tick 94:
//
//   range (T94) → zip_tail (T102) → zip_cdr (T103) → zip_lfh (T104)
//                                                           → **inflate**
//
// PKZIP entries use raw RFC 1951 deflate streams — no zlib header,
// no Adler-32 checksum. `flate2`'s default constructors all assume
// the zlib wrapper, so callers historically tripped on "invalid
// header" errors here. This module exposes one tiny function with
// the right configuration baked in.
//
// Three knobs we expose to keep callers honest:
//
//   * `stored_passthrough` — PKZIP method 0 ("stored") payloads are
//     not deflate-compressed; the right thing is to copy. We surface
//     a helper so callers don't accidentally inflate raw METADATA
//     bytes (which would error).
//
//   * `expected_size` — when the caller knows the uncompressed size
//     from the CDR record (zip_cdr::CdEntry::uncompressed_size), we
//     pre-allocate the output buffer and assert the final length.
//     Mismatch surfaces as a typed error instead of trailing garbage.
//
//   * Hard size cap — wheels are bounded; the largest METADATA we
//     have ever seen in the wild is ~250 KiB (numpy's). Reject
//     payloads claiming more than 8 MiB uncompressed as a zip-bomb
//     guard. Tunable via `inflate_metadata_with_cap`.
//
// Out of scope:
//   * Streaming/incremental decompression — METADATA fits in memory.
//   * CRC verification — caller should compare crc32::checksum(out)
//     against `LocalFileHeader::crc32` or the CDR CRC. Adding it
//     here would tie this module to the `crc32fast` crate, and the
//     PEP 658 fallback path doesn't need the CRC to surface metadata
//     to the resolver.

use flate2::{Decompress, FlushDecompress, Status};

use crate::pkgmanage::pkgmgr::types::IndexError;

/// Default zip-bomb guard — 8 MiB of uncompressed METADATA is more
/// than any wheel in the historical record. Override via
/// [`inflate_metadata_with_cap`] when needed.
pub const DEFAULT_METADATA_CAP: usize = 8 * 1024 * 1024;

/// PKZIP "stored" compression method (APPNOTE.txt §4.4.5). Payload
/// is the raw uncompressed bytes; inflating would corrupt.
pub const METHOD_STORED: u16 = 0;
/// PKZIP "deflated" compression method.
pub const METHOD_DEFLATE: u16 = 8;

/// Inflate a raw RFC 1951 deflate stream into a fresh `Vec<u8>`.
///
/// When `expected_size` is `Some`, the buffer is pre-allocated to
/// that capacity and the final inflated length must match exactly;
/// mismatches return a `ParseError`. When `None`, the buffer grows
/// dynamically and any length the stream happens to produce is
/// accepted (subject to the cap).
pub fn inflate_raw(
    compressed: &[u8],
    expected_size: Option<u64>,
    cap_bytes: usize,
) -> Result<Vec<u8>, IndexError> {
    if let Some(n) = expected_size {
        if (n as usize) > cap_bytes {
            return Err(IndexError::ParseError {
                url: String::new(),
                detail: format!(
                    "expected uncompressed size {n} exceeds cap {cap_bytes} \
                     (raise the cap if this is legitimate)"
                ),
            });
        }
    }

    let initial_capacity = expected_size
        .map(|n| n as usize)
        .unwrap_or(0)
        .min(cap_bytes);
    let mut out = Vec::with_capacity(initial_capacity);

    // `false` = no zlib header (raw deflate as PKZIP uses).
    let mut decoder = Decompress::new(false);

    // Inflate into a scratch buffer in chunks of 64 KiB; each round
    // appends to `out`. We enforce the cap after each append.
    let mut scratch = vec![0u8; 64 * 1024];
    let mut input_pos = 0usize;

    loop {
        let in_before = decoder.total_in();
        let out_before = decoder.total_out();

        let status = decoder
            .decompress(
                &compressed[input_pos..],
                &mut scratch,
                FlushDecompress::None,
            )
            .map_err(|e| IndexError::ParseError {
                url: String::new(),
                detail: format!("deflate decompress error: {e}"),
            })?;

        let produced = (decoder.total_out() - out_before) as usize;
        let consumed = (decoder.total_in() - in_before) as usize;

        if produced > 0 {
            if out.len() + produced > cap_bytes {
                return Err(IndexError::ParseError {
                    url: String::new(),
                    detail: format!(
                        "inflated output would exceed cap {cap_bytes} bytes \
                         (possible zip bomb; raise the cap if legitimate)"
                    ),
                });
            }
            out.extend_from_slice(&scratch[..produced]);
        }
        input_pos += consumed;

        match status {
            Status::StreamEnd => break,
            Status::Ok => {
                // No forward progress + not done → malformed stream.
                if produced == 0 && consumed == 0 {
                    return Err(IndexError::ParseError {
                        url: String::new(),
                        detail: "deflate stream stalled before completion".into(),
                    });
                }
            }
            Status::BufError => {
                // Out of input without a stream-end marker.
                return Err(IndexError::ParseError {
                    url: String::new(),
                    detail: "deflate stream truncated; need more compressed bytes".into(),
                });
            }
        }
    }

    if let Some(n) = expected_size {
        if out.len() as u64 != n {
            return Err(IndexError::ParseError {
                url: String::new(),
                detail: format!(
                    "uncompressed length mismatch: expected {n}, got {}",
                    out.len()
                ),
            });
        }
    }

    Ok(out)
}

/// Dispatch on the PKZIP compression method: copy-through for
/// "stored", inflate for "deflate", reject anything else.
pub fn inflate_metadata(
    method: u16,
    payload: &[u8],
    expected_size: Option<u64>,
) -> Result<Vec<u8>, IndexError> {
    inflate_metadata_with_cap(method, payload, expected_size, DEFAULT_METADATA_CAP)
}

/// Same as [`inflate_metadata`] with an explicit byte cap.
pub fn inflate_metadata_with_cap(
    method: u16,
    payload: &[u8],
    expected_size: Option<u64>,
    cap_bytes: usize,
) -> Result<Vec<u8>, IndexError> {
    match method {
        METHOD_STORED => {
            if payload.len() > cap_bytes {
                return Err(IndexError::ParseError {
                    url: String::new(),
                    detail: format!(
                        "stored payload of {} bytes exceeds cap {cap_bytes}",
                        payload.len()
                    ),
                });
            }
            if let Some(n) = expected_size {
                if payload.len() as u64 != n {
                    return Err(IndexError::ParseError {
                        url: String::new(),
                        detail: format!(
                            "stored payload length mismatch: expected {n}, got {}",
                            payload.len()
                        ),
                    });
                }
            }
            Ok(payload.to_vec())
        }
        METHOD_DEFLATE => inflate_raw(payload, expected_size, cap_bytes),
        other => Err(IndexError::ParseError {
            url: String::new(),
            detail: format!(
                "unsupported PKZIP compression method {other}; \
                 only 0 (stored) and 8 (deflate) are supported for METADATA"
            ),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::write::DeflateEncoder;
    use flate2::Compression;
    use std::io::Write;

    fn err_detail(err: IndexError) -> String {
        match err {
            IndexError::ParseError { detail, .. } => detail,
            other => panic!("expected ParseError, got {other:?}"),
        }
    }

    fn deflate_raw(src: &[u8]) -> Vec<u8> {
        let mut enc = DeflateEncoder::new(Vec::new(), Compression::default());
        enc.write_all(src).unwrap();
        enc.finish().unwrap()
    }

    #[test]
    fn inflate_roundtrip_short_text() {
        let original = b"Metadata-Version: 2.1\nName: example\nVersion: 1.0\n";
        let compressed = deflate_raw(original);
        let result = inflate_raw(
            &compressed,
            Some(original.len() as u64),
            DEFAULT_METADATA_CAP,
        )
        .unwrap();
        assert_eq!(result, original);
    }

    #[test]
    fn inflate_with_no_expected_size() {
        let original = b"hello world";
        let compressed = deflate_raw(original);
        let result = inflate_raw(&compressed, None, DEFAULT_METADATA_CAP).unwrap();
        assert_eq!(result, original);
    }

    #[test]
    fn inflate_repetitive_payload_high_ratio() {
        // ~64 KiB of repeated text — compresses well, exercises
        // the multi-chunk inner loop.
        let original = "ABCDEFGH".repeat(8 * 1024);
        let compressed = deflate_raw(original.as_bytes());
        let result = inflate_raw(
            &compressed,
            Some(original.len() as u64),
            DEFAULT_METADATA_CAP,
        )
        .unwrap();
        assert_eq!(result, original.as_bytes());
    }

    #[test]
    fn inflate_random_size_above_one_chunk() {
        // ~100 KiB of pseudo-random-ish bytes; verifies we keep
        // appending past the 64 KiB scratch boundary.
        let mut original = Vec::with_capacity(100_000);
        for i in 0..100_000 {
            original.push((i % 251) as u8); // 251 is prime — poor compressibility
        }
        let compressed = deflate_raw(&original);
        let result = inflate_raw(
            &compressed,
            Some(original.len() as u64),
            DEFAULT_METADATA_CAP,
        )
        .unwrap();
        assert_eq!(result, original);
    }

    #[test]
    fn inflate_truncated_stream_errors() {
        let original = b"some content that needs more than 4 bytes";
        let compressed = deflate_raw(original);
        let truncated = &compressed[..compressed.len() / 2];
        let err = inflate_raw(truncated, None, DEFAULT_METADATA_CAP).unwrap_err();
        let detail = err_detail(err);
        assert!(detail.contains("truncated") || detail.contains("error"));
    }

    #[test]
    fn inflate_garbage_input_errors() {
        let garbage = [0xFF, 0xFE, 0xFD, 0xFC, 0xFB, 0xFA];
        let err = inflate_raw(&garbage, None, DEFAULT_METADATA_CAP).unwrap_err();
        let detail = err_detail(err);
        assert!(detail.contains("error") || detail.contains("stalled"));
    }

    #[test]
    fn inflate_size_mismatch_errors() {
        let original = b"actual content";
        let compressed = deflate_raw(original);
        // Lie about the size.
        let err = inflate_raw(&compressed, Some(9999), DEFAULT_METADATA_CAP).unwrap_err();
        assert!(err_detail(err).contains("length mismatch"));
    }

    #[test]
    fn inflate_rejects_expected_size_above_cap() {
        let err = inflate_raw(&[], Some(100), 50).unwrap_err();
        assert!(err_detail(err).contains("exceeds cap"));
    }

    #[test]
    fn inflate_rejects_output_overrun_cap() {
        // Compress 10 KiB, set cap to 1 KiB → must reject.
        let original = vec![b'x'; 10 * 1024];
        let compressed = deflate_raw(&original);
        let err = inflate_raw(&compressed, None, 1024).unwrap_err();
        assert!(err_detail(err).contains("exceed cap"));
    }

    #[test]
    fn inflate_metadata_stored_passthrough() {
        let payload = b"Metadata-Version: 2.1\nName: stored-pkg\n";
        let out = inflate_metadata(METHOD_STORED, payload, Some(payload.len() as u64)).unwrap();
        assert_eq!(out, payload);
    }

    #[test]
    fn inflate_metadata_stored_length_mismatch_errors() {
        let payload = b"actual bytes";
        let err = inflate_metadata(METHOD_STORED, payload, Some(99)).unwrap_err();
        assert!(err_detail(err).contains("stored payload length mismatch"));
    }

    #[test]
    fn inflate_metadata_stored_overrun_cap() {
        let payload = vec![b'x'; 1000];
        let err = inflate_metadata_with_cap(METHOD_STORED, &payload, None, 100).unwrap_err();
        assert!(err_detail(err).contains("exceeds cap"));
    }

    #[test]
    fn inflate_metadata_deflate_method() {
        let original = b"Metadata-Version: 2.1\nName: deflated-pkg\n";
        let compressed = deflate_raw(original);
        let out =
            inflate_metadata(METHOD_DEFLATE, &compressed, Some(original.len() as u64)).unwrap();
        assert_eq!(out, original);
    }

    #[test]
    fn inflate_metadata_rejects_unsupported_method() {
        // Method 12 = bzip2 (legal PKZIP but not wheel-canonical).
        let err = inflate_metadata(12, &[1, 2, 3], None).unwrap_err();
        let detail = err_detail(err);
        assert!(detail.contains("unsupported PKZIP compression method 12"));
        assert!(detail.contains("only 0 (stored) and 8 (deflate)"));
    }

    #[test]
    fn empty_input_with_zero_expected_size() {
        // Edge case: zero-length stored payload, expected = 0 → OK.
        let out = inflate_metadata(METHOD_STORED, &[], Some(0)).unwrap();
        assert!(out.is_empty());
    }

    #[test]
    fn realistic_wheel_metadata_roundtrip() {
        let metadata = "\
Metadata-Version: 2.1
Name: requests
Version: 2.31.0
Summary: Python HTTP for Humans.
Home-page: https://requests.readthedocs.io
Author: Kenneth Reitz
Author-email: me@kennethreitz.org
License: Apache 2.0
Requires-Python: >=3.7
Requires-Dist: charset-normalizer (>=2,<4)
Requires-Dist: idna (>=2.5,<4)
Requires-Dist: urllib3 (>=1.21.1,<3)
Requires-Dist: certifi (>=2017.4.17)
";
        let compressed = deflate_raw(metadata.as_bytes());
        // Verify the realistic ratio actually compresses (sanity check
        // on the fixture itself).
        assert!(compressed.len() < metadata.len());

        let out =
            inflate_metadata(METHOD_DEFLATE, &compressed, Some(metadata.len() as u64)).unwrap();
        assert_eq!(out, metadata.as_bytes());

        // Smoke-test the first line so the test fails loudly if we
        // ever silently produce empty output.
        let head = String::from_utf8_lossy(&out);
        assert!(head.starts_with("Metadata-Version: 2.1"));
    }

    #[test]
    fn default_cap_is_eight_mib() {
        assert_eq!(DEFAULT_METADATA_CAP, 8 * 1024 * 1024);
    }

    #[test]
    fn method_constants_match_spec() {
        assert_eq!(METHOD_STORED, 0);
        assert_eq!(METHOD_DEFLATE, 8);
    }

    #[test]
    fn cap_boundary_exactly_at_limit_is_ok() {
        let original = vec![b'y'; 1024];
        let compressed = deflate_raw(&original);
        // Cap == exact size → must succeed.
        let out = inflate_raw(&compressed, Some(1024), 1024).unwrap();
        assert_eq!(out.len(), 1024);
    }
}

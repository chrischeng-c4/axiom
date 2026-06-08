// zip_cdr.rs — PKZIP Central Directory entry walker.
//
// Step 3 of the PEP 658 fallback flow for plucking `METADATA` out of
// a remote wheel without downloading the whole archive:
//
//   1. HEAD/GET with `Range:` for the last ~64 KiB (range.rs, Tick 94).
//   2. parse_zip_tail → locate the Central Directory (zip_tail.rs, Tick 102).
//   3. **walk_central_directory** → enumerate entries, find the one
//      whose filename ends in `.dist-info/METADATA`.
//   4. Issue a second Range GET for that entry's local file header +
//      compressed bytes, then inflate.
//
// References:
//   * PKZIP APPNOTE.txt §4.3.12 — Central Directory file headers.
//   * APPNOTE.txt §4.5 — extra field layout, Zip64 (id 0x0001) extension.
//   * APPNOTE.txt §4.4.5 — compression method codes (0 = stored, 8 = deflate).
//
// Out of scope:
//   * Decoding `extra` records other than Zip64.
//   * Filename decoding under the "language encoding flag" (bit 11):
//     UTF-8 vs CP437. We parse bytes as UTF-8 (lossy) since modern wheel
//     producers always emit UTF-8; CP437-only legacy filenames would
//     surface as mojibake, but a wheel is required by PEP 427 to use
//     UTF-8 filenames anyway, so this is a non-issue in practice.

use crate::pkgmanage::pkgmgr::types::IndexError;

/// PK\01\02 — Central Directory file header signature.
const CDR_SIG: [u8; 4] = [0x50, 0x4b, 0x01, 0x02];
/// 46 bytes of fixed-width header before variable filename/extra/comment.
const CDR_FIXED_LEN: usize = 46;
/// Zip64 extended-information extra field tag.
const ZIP64_EXTRA_ID: u16 = 0x0001;
/// Sentinel placed in 32-bit size/offset fields when the real value is
/// in the Zip64 extra field.
const ZIP64_SENTINEL_U32: u32 = 0xFFFF_FFFF;
/// Disk-number sentinel; we don't process disk numbers, but the
/// constant is referenced in tests + acts as a defensive anchor
/// against accidental edits to the surrounding constants block.
#[allow(dead_code)]
const ZIP64_SENTINEL_U16: u16 = 0xFFFF;

/// One Central Directory record decoded into the fields we care about
/// for the PEP 658 fallback. Other fields (CRC-32, file attributes,
/// disk number, etc.) are intentionally omitted — they don't influence
/// the next Range request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CdEntry {
    /// Filename as stored in the CD record. Forward-slashes per spec.
    pub filename: String,
    /// Offset of the entry's local file header from the start of the
    /// archive. This is the byte offset to seek to for the next Range
    /// request.
    pub local_header_offset: u64,
    /// Compressed (on-disk) size of the entry payload.
    pub compressed_size: u64,
    /// Uncompressed (original) size — useful for sanity checking the
    /// inflate output.
    pub uncompressed_size: u64,
    /// PKZIP compression method. 0 = stored, 8 = deflate. Wheels are
    /// almost always 8.
    pub compression_method: u16,
}

impl CdEntry {
    /// True iff the filename ends with `.dist-info/METADATA`. This is
    /// the exact suffix the wheel spec (PEP 427 §"The .dist-info
    /// directory") guarantees for the metadata file inside a wheel.
    pub fn is_metadata(&self) -> bool {
        self.filename.ends_with(".dist-info/METADATA")
    }
}

/// Walk the Central Directory bytes returned by step 2 (the
/// `[cd_offset, cd_offset + cd_size)` slice). Returns one [`CdEntry`]
/// per record in declaration order.
///
/// Errors on truncated records or unknown signatures. Trailing
/// padding / bytes after the last record are tolerated.
pub fn walk_central_directory(cd_bytes: &[u8]) -> Result<Vec<CdEntry>, IndexError> {
    let mut entries = Vec::new();
    let mut cursor = 0usize;

    while cursor + 4 <= cd_bytes.len() {
        // Stop cleanly on EOCD or Zip64 EOCD locator/record signatures —
        // some producers concatenate the CD and EOCD in one Range
        // response (we'd be handed the whole tail blob).
        let sig = &cd_bytes[cursor..cursor + 4];
        if sig != CDR_SIG {
            if sig == [0x50, 0x4b, 0x05, 0x06]      // EOCD
                || sig == [0x50, 0x4b, 0x06, 0x06]  // Zip64 EOCD
                || sig == [0x50, 0x4b, 0x06, 0x07]  // Zip64 locator
            {
                break;
            }
            return Err(IndexError::ParseError {
                url: String::new(),
                detail: format!(
                    "unexpected signature {:02x?} in Central Directory at offset {cursor}",
                    sig
                ),
            });
        }

        if cursor + CDR_FIXED_LEN > cd_bytes.len() {
            return Err(IndexError::ParseError {
                url: String::new(),
                detail: "Central Directory record truncated (fixed header)".into(),
            });
        }

        let hdr = &cd_bytes[cursor..cursor + CDR_FIXED_LEN];
        let compression_method = u16_le(&hdr[10..12]);
        let compressed_size_32 = u32_le(&hdr[20..24]);
        let uncompressed_size_32 = u32_le(&hdr[24..28]);
        let filename_len = u16_le(&hdr[28..30]) as usize;
        let extra_len = u16_le(&hdr[30..32]) as usize;
        let comment_len = u16_le(&hdr[32..34]) as usize;
        let local_offset_32 = u32_le(&hdr[42..46]);

        let var_start = cursor + CDR_FIXED_LEN;
        let var_end = var_start
            .checked_add(filename_len)
            .and_then(|n| n.checked_add(extra_len))
            .and_then(|n| n.checked_add(comment_len))
            .ok_or_else(|| IndexError::ParseError {
                url: String::new(),
                detail: "Central Directory record length overflow".into(),
            })?;
        if var_end > cd_bytes.len() {
            return Err(IndexError::ParseError {
                url: String::new(),
                detail: "Central Directory record truncated (variable fields)".into(),
            });
        }

        let filename_bytes = &cd_bytes[var_start..var_start + filename_len];
        let extra_bytes =
            &cd_bytes[var_start + filename_len..var_start + filename_len + extra_len];

        let filename = String::from_utf8_lossy(filename_bytes).into_owned();

        // Zip64 promotion: if any of size/offset fields are the
        // 0xFFFFFFFF sentinel, the real 64-bit values live in the
        // Zip64 extra field. Per APPNOTE.txt §4.5.3 the field order
        // follows the same order as the sentinelled fields: original
        // size, then compressed size, then local header offset, then
        // disk start number.
        let (uncompressed_size, compressed_size, local_header_offset) = promote_zip64(
            extra_bytes,
            uncompressed_size_32,
            compressed_size_32,
            local_offset_32,
        )?;

        entries.push(CdEntry {
            filename,
            local_header_offset,
            compressed_size,
            uncompressed_size,
            compression_method,
        });

        cursor = var_end;
    }

    Ok(entries)
}

/// Pluck the first entry whose filename ends in `.dist-info/METADATA`.
/// A well-formed wheel has exactly one such entry; if a malformed
/// archive has multiple, callers get the first one in CD declaration
/// order — same as `unzip -p`.
pub fn find_metadata_entry(entries: &[CdEntry]) -> Option<&CdEntry> {
    entries.iter().find(|e| e.is_metadata())
}

fn u16_le(bytes: &[u8]) -> u16 {
    u16::from_le_bytes([bytes[0], bytes[1]])
}

fn u32_le(bytes: &[u8]) -> u32 {
    u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
}

fn u64_le(bytes: &[u8]) -> u64 {
    let mut buf = [0u8; 8];
    buf.copy_from_slice(&bytes[..8]);
    u64::from_le_bytes(buf)
}

fn promote_zip64(
    extra: &[u8],
    uncompressed_32: u32,
    compressed_32: u32,
    local_offset_32: u32,
) -> Result<(u64, u64, u64), IndexError> {
    let mut uncompressed = uncompressed_32 as u64;
    let mut compressed = compressed_32 as u64;
    let mut local_offset = local_offset_32 as u64;

    let needs_uncompressed = uncompressed_32 == ZIP64_SENTINEL_U32;
    let needs_compressed = compressed_32 == ZIP64_SENTINEL_U32;
    let needs_offset = local_offset_32 == ZIP64_SENTINEL_U32;

    if !(needs_uncompressed || needs_compressed || needs_offset) {
        return Ok((uncompressed, compressed, local_offset));
    }

    let mut cursor = 0usize;
    while cursor + 4 <= extra.len() {
        let id = u16_le(&extra[cursor..cursor + 2]);
        let size = u16_le(&extra[cursor + 2..cursor + 4]) as usize;
        let body_start = cursor + 4;
        let body_end = body_start.checked_add(size).ok_or_else(|| IndexError::ParseError {
            url: String::new(),
            detail: "Zip64 extra field length overflow".into(),
        })?;
        if body_end > extra.len() {
            return Err(IndexError::ParseError {
                url: String::new(),
                detail: "Zip64 extra field truncated".into(),
            });
        }

        if id == ZIP64_EXTRA_ID {
            let body = &extra[body_start..body_end];
            let mut field_cursor = 0usize;
            for (needed, slot) in [
                (needs_uncompressed, &mut uncompressed),
                (needs_compressed, &mut compressed),
                (needs_offset, &mut local_offset),
            ] {
                if !needed {
                    continue;
                }
                if field_cursor + 8 > body.len() {
                    return Err(IndexError::ParseError {
                        url: String::new(),
                        detail: "Zip64 extra field missing required 64-bit slot".into(),
                    });
                }
                *slot = u64_le(&body[field_cursor..field_cursor + 8]);
                field_cursor += 8;
            }
            return Ok((uncompressed, compressed, local_offset));
        }

        cursor = body_end;
    }

    // Sentinel present but no Zip64 extra field found — that's a
    // malformed archive, not just a missing optional field.
    if needs_uncompressed || needs_compressed || needs_offset {
        return Err(IndexError::ParseError {
            url: String::new(),
            detail: "Central Directory sentinel value with no Zip64 extra field".into(),
        });
    }

    Ok((uncompressed, compressed, local_offset))
}

#[cfg(test)]
#[allow(clippy::identity_op)]
mod tests {
    use super::*;

    fn err_detail(err: IndexError) -> String {
        match err {
            IndexError::ParseError { detail, .. } => detail,
            other => panic!("expected ParseError, got {other:?}"),
        }
    }

    /// Build one CDR record. `local_offset` is the 32-bit value to
    /// write (use `ZIP64_SENTINEL_U32` to force Zip64 promotion).
    fn build_cdr(
        filename: &[u8],
        compressed_size: u32,
        uncompressed_size: u32,
        local_offset: u32,
        compression_method: u16,
        extra: &[u8],
    ) -> Vec<u8> {
        let mut buf = Vec::with_capacity(CDR_FIXED_LEN + filename.len() + extra.len());
        buf.extend_from_slice(&CDR_SIG);
        buf.extend_from_slice(&20u16.to_le_bytes());      // version made by
        buf.extend_from_slice(&20u16.to_le_bytes());      // version needed
        buf.extend_from_slice(&0u16.to_le_bytes());       // gp flag
        buf.extend_from_slice(&compression_method.to_le_bytes()); // method
        buf.extend_from_slice(&0u16.to_le_bytes());       // mod time
        buf.extend_from_slice(&0u16.to_le_bytes());       // mod date
        buf.extend_from_slice(&0u32.to_le_bytes());       // CRC-32
        buf.extend_from_slice(&compressed_size.to_le_bytes());
        buf.extend_from_slice(&uncompressed_size.to_le_bytes());
        buf.extend_from_slice(&(filename.len() as u16).to_le_bytes());
        buf.extend_from_slice(&(extra.len() as u16).to_le_bytes());
        buf.extend_from_slice(&0u16.to_le_bytes());       // comment len
        buf.extend_from_slice(&0u16.to_le_bytes());       // disk number
        buf.extend_from_slice(&0u16.to_le_bytes());       // internal attrs
        buf.extend_from_slice(&0u32.to_le_bytes());       // external attrs
        buf.extend_from_slice(&local_offset.to_le_bytes());
        buf.extend_from_slice(filename);
        buf.extend_from_slice(extra);
        buf
    }

    #[test]
    fn walk_single_deflate_entry() {
        let cd = build_cdr(b"pkg-1.0.dist-info/METADATA", 512, 1024, 100, 8, &[]);
        let entries = walk_central_directory(&cd).unwrap();
        assert_eq!(entries.len(), 1);
        let e = &entries[0];
        assert_eq!(e.filename, "pkg-1.0.dist-info/METADATA");
        assert_eq!(e.compressed_size, 512);
        assert_eq!(e.uncompressed_size, 1024);
        assert_eq!(e.local_header_offset, 100);
        assert_eq!(e.compression_method, 8);
        assert!(e.is_metadata());
    }

    #[test]
    fn walk_stored_entry() {
        let cd = build_cdr(b"pkg-1.0.dist-info/WHEEL", 256, 256, 0, 0, &[]);
        let entries = walk_central_directory(&cd).unwrap();
        assert_eq!(entries[0].compression_method, 0);
        assert!(!entries[0].is_metadata());
    }

    #[test]
    fn walk_multiple_entries_in_declaration_order() {
        let mut cd = build_cdr(b"pkg/__init__.py", 50, 80, 0, 8, &[]);
        cd.extend(build_cdr(b"pkg-1.0.dist-info/RECORD", 200, 300, 500, 8, &[]));
        cd.extend(build_cdr(b"pkg-1.0.dist-info/METADATA", 400, 600, 800, 8, &[]));

        let entries = walk_central_directory(&cd).unwrap();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].filename, "pkg/__init__.py");
        assert_eq!(entries[1].filename, "pkg-1.0.dist-info/RECORD");
        assert_eq!(entries[2].filename, "pkg-1.0.dist-info/METADATA");

        let meta = find_metadata_entry(&entries).unwrap();
        assert_eq!(meta.filename, "pkg-1.0.dist-info/METADATA");
        assert_eq!(meta.local_header_offset, 800);
    }

    #[test]
    fn find_metadata_returns_none_when_absent() {
        let cd = build_cdr(b"pkg/__init__.py", 50, 80, 0, 8, &[]);
        let entries = walk_central_directory(&cd).unwrap();
        assert!(find_metadata_entry(&entries).is_none());
    }

    #[test]
    fn find_metadata_prefers_first_match() {
        // Pathological: two METADATA entries. unzip -p uses the first.
        let mut cd = build_cdr(b"a-1.0.dist-info/METADATA", 100, 200, 10, 8, &[]);
        cd.extend(build_cdr(b"b-1.0.dist-info/METADATA", 100, 200, 20, 8, &[]));
        let entries = walk_central_directory(&cd).unwrap();
        let meta = find_metadata_entry(&entries).unwrap();
        assert_eq!(meta.filename, "a-1.0.dist-info/METADATA");
        assert_eq!(meta.local_header_offset, 10);
    }

    #[test]
    fn metadata_suffix_matching_is_strict() {
        // "METADATA.json" must NOT match — only the canonical suffix
        // counts.
        let cd = build_cdr(b"pkg-1.0.dist-info/METADATA.json", 100, 200, 0, 8, &[]);
        let entries = walk_central_directory(&cd).unwrap();
        assert!(!entries[0].is_metadata());
    }

    #[test]
    fn empty_input_yields_no_entries() {
        let entries = walk_central_directory(&[]).unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn eocd_signature_terminates_walk_cleanly() {
        let mut blob = build_cdr(b"pkg-1.0.dist-info/METADATA", 50, 80, 0, 8, &[]);
        // EOCD signature + a few bytes — should stop, not error.
        blob.extend_from_slice(&[0x50, 0x4b, 0x05, 0x06, 0, 0, 0, 0]);
        let entries = walk_central_directory(&blob).unwrap();
        assert_eq!(entries.len(), 1);
    }

    #[test]
    fn zip64_eocd_signature_terminates_walk_cleanly() {
        let mut blob = build_cdr(b"pkg-1.0.dist-info/METADATA", 50, 80, 0, 8, &[]);
        blob.extend_from_slice(&[0x50, 0x4b, 0x06, 0x06]);
        let entries = walk_central_directory(&blob).unwrap();
        assert_eq!(entries.len(), 1);
    }

    #[test]
    fn zip64_locator_signature_terminates_walk_cleanly() {
        let mut blob = build_cdr(b"pkg-1.0.dist-info/METADATA", 50, 80, 0, 8, &[]);
        blob.extend_from_slice(&[0x50, 0x4b, 0x06, 0x07]);
        let entries = walk_central_directory(&blob).unwrap();
        assert_eq!(entries.len(), 1);
    }

    #[test]
    fn unknown_signature_is_an_error() {
        let mut bad = Vec::from(&[0xDE, 0xAD, 0xBE, 0xEF][..]);
        bad.extend(build_cdr(b"x", 0, 0, 0, 0, &[]));
        let err = walk_central_directory(&bad).unwrap_err();
        assert!(err_detail(err).contains("unexpected signature"));
    }

    #[test]
    fn truncated_fixed_header_errors() {
        let cd = build_cdr(b"pkg-1.0.dist-info/METADATA", 50, 80, 0, 8, &[]);
        let truncated = &cd[..CDR_FIXED_LEN - 1];
        let err = walk_central_directory(truncated).unwrap_err();
        assert!(err_detail(err).contains("truncated (fixed header)"));
    }

    #[test]
    fn truncated_variable_fields_errors() {
        let cd = build_cdr(b"pkg-1.0.dist-info/METADATA", 50, 80, 0, 8, &[]);
        // Lop off some filename bytes.
        let truncated = &cd[..cd.len() - 5];
        let err = walk_central_directory(truncated).unwrap_err();
        assert!(err_detail(err).contains("truncated (variable fields)"));
    }

    // ---- Zip64 promotion ---------------------------------------------

    /// Build a Zip64 extra field with the requested 64-bit values, in
    /// the order APPNOTE.txt §4.5.3 prescribes: uncompressed, then
    /// compressed, then local-header-offset.
    fn build_zip64_extra(values: &[u64]) -> Vec<u8> {
        let mut buf = Vec::with_capacity(4 + values.len() * 8);
        buf.extend_from_slice(&ZIP64_EXTRA_ID.to_le_bytes());
        buf.extend_from_slice(&((values.len() * 8) as u16).to_le_bytes());
        for v in values {
            buf.extend_from_slice(&v.to_le_bytes());
        }
        buf
    }

    #[test]
    fn zip64_promotion_for_offset_only() {
        // 32-bit sizes are real; offset sentinel forces Zip64 lookup.
        let extra = build_zip64_extra(&[5_000_000_000]);
        let cd = build_cdr(
            b"big.dist-info/METADATA",
            512,
            1024,
            ZIP64_SENTINEL_U32,
            8,
            &extra,
        );
        let entries = walk_central_directory(&cd).unwrap();
        assert_eq!(entries[0].compressed_size, 512);
        assert_eq!(entries[0].uncompressed_size, 1024);
        assert_eq!(entries[0].local_header_offset, 5_000_000_000);
    }

    #[test]
    fn zip64_promotion_for_all_three_fields() {
        let extra = build_zip64_extra(&[
            10_000_000_000, // uncompressed
            5_000_000_000,  // compressed
            8_000_000_000,  // local offset
        ]);
        let cd = build_cdr(
            b"huge.dist-info/METADATA",
            ZIP64_SENTINEL_U32,
            ZIP64_SENTINEL_U32,
            ZIP64_SENTINEL_U32,
            8,
            &extra,
        );
        let entries = walk_central_directory(&cd).unwrap();
        assert_eq!(entries[0].uncompressed_size, 10_000_000_000);
        assert_eq!(entries[0].compressed_size, 5_000_000_000);
        assert_eq!(entries[0].local_header_offset, 8_000_000_000);
    }

    #[test]
    fn zip64_extra_with_other_tags_first() {
        // Some producers prepend a unicode-path (id 0x7075) or
        // NTFS (id 0x000a) extra block. Make sure we skip past them.
        let mut extra = Vec::new();
        extra.extend_from_slice(&0x7075u16.to_le_bytes()); // unicode path
        extra.extend_from_slice(&4u16.to_le_bytes());
        extra.extend_from_slice(&[0xDE, 0xAD, 0xBE, 0xEF]);
        extra.extend_from_slice(&build_zip64_extra(&[6_000_000_000]));

        let cd = build_cdr(
            b"big.dist-info/METADATA",
            100,
            200,
            ZIP64_SENTINEL_U32,
            8,
            &extra,
        );
        let entries = walk_central_directory(&cd).unwrap();
        assert_eq!(entries[0].local_header_offset, 6_000_000_000);
    }

    #[test]
    fn zip64_sentinel_without_extra_field_errors() {
        let cd = build_cdr(
            b"bad.dist-info/METADATA",
            100,
            200,
            ZIP64_SENTINEL_U32,
            8,
            &[],
        );
        let err = walk_central_directory(&cd).unwrap_err();
        assert!(err_detail(err).contains("sentinel value with no Zip64"));
    }

    #[test]
    fn zip64_extra_too_small_for_required_slots_errors() {
        // Sentinel says we need 24 bytes (3 × u64); extra only ships 8.
        let extra = build_zip64_extra(&[1]);
        let cd = build_cdr(
            b"bad.dist-info/METADATA",
            ZIP64_SENTINEL_U32,
            ZIP64_SENTINEL_U32,
            ZIP64_SENTINEL_U32,
            8,
            &extra,
        );
        let err = walk_central_directory(&cd).unwrap_err();
        assert!(err_detail(err).contains("missing required 64-bit slot"));
    }

    #[test]
    fn zip64_extra_truncated_errors() {
        // Tag claims 16 bytes but only 4 are present.
        let mut extra = Vec::new();
        extra.extend_from_slice(&ZIP64_EXTRA_ID.to_le_bytes());
        extra.extend_from_slice(&16u16.to_le_bytes());
        extra.extend_from_slice(&[0, 0, 0, 0]);

        let cd = build_cdr(
            b"bad.dist-info/METADATA",
            ZIP64_SENTINEL_U32,
            0,
            0,
            8,
            &extra,
        );
        let err = walk_central_directory(&cd).unwrap_err();
        assert!(err_detail(err).contains("Zip64 extra field truncated"));
    }

    #[test]
    fn non_zip64_extras_are_ignored_without_sentinels() {
        // No sentinels → extra field never consulted.
        let mut extra = Vec::new();
        extra.extend_from_slice(&0x000au16.to_le_bytes()); // NTFS
        extra.extend_from_slice(&8u16.to_le_bytes());
        extra.extend_from_slice(&[0; 8]);

        let cd = build_cdr(b"pkg.dist-info/METADATA", 100, 200, 50, 8, &extra);
        let entries = walk_central_directory(&cd).unwrap();
        assert_eq!(entries[0].compressed_size, 100);
        assert_eq!(entries[0].local_header_offset, 50);
    }

    #[test]
    fn realistic_wheel_layout_walks_clean() {
        // Mock a small wheel CD: __init__.py, RECORD, WHEEL, METADATA.
        let mut cd = Vec::new();
        cd.extend(build_cdr(b"pkg/__init__.py", 30, 50, 0, 8, &[]));
        cd.extend(build_cdr(b"pkg-1.0.dist-info/RECORD", 200, 300, 100, 8, &[]));
        cd.extend(build_cdr(b"pkg-1.0.dist-info/WHEEL", 80, 100, 500, 8, &[]));
        cd.extend(build_cdr(b"pkg-1.0.dist-info/METADATA", 1024, 2048, 700, 8, &[]));

        let entries = walk_central_directory(&cd).unwrap();
        assert_eq!(entries.len(), 4);

        let meta = find_metadata_entry(&entries).unwrap();
        assert_eq!(meta.local_header_offset, 700);
        assert_eq!(meta.compressed_size, 1024);
        assert_eq!(meta.uncompressed_size, 2048);
        assert_eq!(meta.compression_method, 8);
    }

    #[test]
    fn extra_field_zero_length_is_ok() {
        // size=0 extra field record (legal per spec) — must not loop.
        let mut extra = Vec::new();
        extra.extend_from_slice(&0x9999u16.to_le_bytes()); // unknown id
        extra.extend_from_slice(&0u16.to_le_bytes());      // size 0
        extra.extend_from_slice(&build_zip64_extra(&[42]));

        let cd = build_cdr(
            b"pkg.dist-info/METADATA",
            100,
            200,
            ZIP64_SENTINEL_U32,
            8,
            &extra,
        );
        let entries = walk_central_directory(&cd).unwrap();
        assert_eq!(entries[0].local_header_offset, 42);
    }

    #[test]
    fn zip64_sentinel_u16_constant_is_max_u16() {
        // Sanity: the disk-number sentinel value is u16::MAX.
        // (Not used by our parser, but documented in the constants
        // block — defends against accidental edits.)
        assert_eq!(ZIP64_SENTINEL_U16, u16::MAX);
    }
}

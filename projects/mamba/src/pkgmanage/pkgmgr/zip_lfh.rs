// zip_lfh.rs — PKZIP Local File Header parser.
//
// Step 4 (and final) of the PEP 658 fallback flow for plucking
// `METADATA` out of a remote wheel without downloading the whole
// archive:
//
//   1. HEAD/GET with `Range:` for the last ~64 KiB (range.rs, Tick 94).
//   2. parse_zip_tail → locate the Central Directory (zip_tail.rs, T102).
//   3. walk_central_directory → find the METADATA entry, get its
//      `local_header_offset` + `compressed_size` (zip_cdr.rs, T103).
//   4. **parse_lfh** → parse the 30-byte Local File Header at that
//      offset so we know where the compressed payload starts.
//   5. Inflate the payload bytes [payload_offset, payload_offset +
//      compressed_size) (out of scope here; that's a `flate2`
//      crate dependency).
//
// References:
//   * PKZIP APPNOTE.txt §4.3.7 — Local File Header signature + fixed
//     30-byte fields + variable filename + extra.
//   * §4.4.4 general-purpose bit flag 3 — "data descriptor follows"
//     (legacy streaming writers leave compressed_size / CRC zeroed in
//     the LFH and append a Data Descriptor record after the payload).
//
// Out of scope:
//   * Decoding the payload (caller decompresses with flate2 etc.).
//   * Data-descriptor recovery — wheels are produced from a complete
//     buffer and never stream, so bit 3 is always 0. If we ever see
//     bit 3 set, the caller should fall back to the CDR-supplied
//     `compressed_size` (which we expose verbatim, so this is a no-op
//     from a fetch perspective; only the CRC differs).

use crate::pkgmanage::pkgmgr::types::IndexError;

/// PK\03\04 — Local File Header signature.
const LFH_SIG: [u8; 4] = [0x50, 0x4b, 0x03, 0x04];
/// 30 bytes of fixed-width header before variable filename + extra.
const LFH_FIXED_LEN: usize = 30;

/// The handful of LFH fields a caller actually needs in order to
/// compute where the entry payload begins and how many compressed
/// bytes to ask for. Other fields (mod time/date, version-needed,
/// general-purpose bit flag) are intentionally dropped.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalFileHeader {
    /// Filename copied out of the variable-length region. Should
    /// match the CDR `CdEntry::filename` — callers should assert this.
    pub filename: String,
    /// Compressed size as recorded in the LFH itself. **Zero** when
    /// the general-purpose bit-flag 3 ("data descriptor") is set, in
    /// which case callers must use the CDR-supplied size instead.
    pub compressed_size: u32,
    /// Total bytes consumed by the header (signature + fixed + variable).
    /// Add this to the offset at which the LFH bytes were fetched to
    /// get the offset of the compressed payload.
    pub header_size: u64,
    /// CRC-32 of the uncompressed data, or 0 when bit 3 is set.
    pub crc32: u32,
    /// PKZIP compression method (0 = stored, 8 = deflate).
    pub compression_method: u16,
    /// General-purpose bit flag, exposed for callers that care about
    /// bit 3 (data descriptor) or bit 0 (encryption — wheels never
    /// encrypt, so callers should reject non-zero unconditionally).
    pub gp_flag: u16,
}

impl LocalFileHeader {
    /// True iff the entry uses a streaming "data descriptor" (bit 3).
    /// When true, `compressed_size` / `crc32` in the LFH are zeroed —
    /// callers must read those values from the Central Directory.
    pub fn has_data_descriptor(&self) -> bool {
        (self.gp_flag & 0x0008) != 0
    }

    /// True iff the entry is encrypted (bit 0). Wheels are never
    /// encrypted; surface this so the caller can refuse cleanly.
    pub fn is_encrypted(&self) -> bool {
        (self.gp_flag & 0x0001) != 0
    }
}

/// Parse a Local File Header from the start of `bytes`. The caller
/// is responsible for fetching at least `LFH_FIXED_LEN + filename_len
/// + extra_len` bytes — when the input is too short to read the
/// variable portion, we return an error suggesting they pad their
/// Range request.
pub fn parse_lfh(bytes: &[u8]) -> Result<LocalFileHeader, IndexError> {
    if bytes.len() < LFH_FIXED_LEN {
        return Err(IndexError::ParseError {
            url: String::new(),
            detail: format!(
                "Local File Header truncated: need >= {LFH_FIXED_LEN} bytes, got {}",
                bytes.len()
            ),
        });
    }
    if bytes[..4] != LFH_SIG {
        return Err(IndexError::ParseError {
            url: String::new(),
            detail: format!(
                "expected Local File Header signature PK\\03\\04, got {:02x?}",
                &bytes[..4]
            ),
        });
    }

    let gp_flag = u16_le(&bytes[6..8]);
    let compression_method = u16_le(&bytes[8..10]);
    let crc32 = u32_le(&bytes[14..18]);
    let compressed_size = u32_le(&bytes[18..22]);
    let filename_len = u16_le(&bytes[26..28]) as usize;
    let extra_len = u16_le(&bytes[28..30]) as usize;

    let var_end = LFH_FIXED_LEN
        .checked_add(filename_len)
        .and_then(|n| n.checked_add(extra_len))
        .ok_or_else(|| IndexError::ParseError {
            url: String::new(),
            detail: "Local File Header length overflow".into(),
        })?;
    if var_end > bytes.len() {
        return Err(IndexError::ParseError {
            url: String::new(),
            detail: format!(
                "Local File Header variable region truncated: need {var_end} bytes, got {}; \
                 fetch a larger window covering filename ({filename_len}) + extra ({extra_len})",
                bytes.len()
            ),
        });
    }

    let filename =
        String::from_utf8_lossy(&bytes[LFH_FIXED_LEN..LFH_FIXED_LEN + filename_len]).into_owned();

    Ok(LocalFileHeader {
        filename,
        compressed_size,
        header_size: var_end as u64,
        crc32,
        compression_method,
        gp_flag,
    })
}

fn u16_le(bytes: &[u8]) -> u16 {
    u16::from_le_bytes([bytes[0], bytes[1]])
}

fn u32_le(bytes: &[u8]) -> u32 {
    u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn err_detail(err: IndexError) -> String {
        match err {
            IndexError::ParseError { detail, .. } => detail,
            other => panic!("expected ParseError, got {other:?}"),
        }
    }

    /// Build a Local File Header with the requested fields.
    fn build_lfh(
        filename: &[u8],
        compressed_size: u32,
        uncompressed_size: u32,
        compression_method: u16,
        gp_flag: u16,
        crc32: u32,
        extra: &[u8],
    ) -> Vec<u8> {
        let mut buf = Vec::with_capacity(LFH_FIXED_LEN + filename.len() + extra.len());
        buf.extend_from_slice(&LFH_SIG);
        buf.extend_from_slice(&20u16.to_le_bytes()); // version needed
        buf.extend_from_slice(&gp_flag.to_le_bytes());
        buf.extend_from_slice(&compression_method.to_le_bytes());
        buf.extend_from_slice(&0u16.to_le_bytes()); // mod time
        buf.extend_from_slice(&0u16.to_le_bytes()); // mod date
        buf.extend_from_slice(&crc32.to_le_bytes());
        buf.extend_from_slice(&compressed_size.to_le_bytes());
        buf.extend_from_slice(&uncompressed_size.to_le_bytes());
        buf.extend_from_slice(&(filename.len() as u16).to_le_bytes());
        buf.extend_from_slice(&(extra.len() as u16).to_le_bytes());
        buf.extend_from_slice(filename);
        buf.extend_from_slice(extra);
        buf
    }

    #[test]
    fn parse_basic_deflate_header() {
        let lfh = build_lfh(
            b"pkg-1.0.dist-info/METADATA",
            512,
            1024,
            8,
            0,
            0xDEADBEEF,
            &[],
        );
        let parsed = parse_lfh(&lfh).unwrap();
        assert_eq!(parsed.filename, "pkg-1.0.dist-info/METADATA");
        assert_eq!(parsed.compressed_size, 512);
        assert_eq!(parsed.compression_method, 8);
        assert_eq!(parsed.gp_flag, 0);
        assert_eq!(parsed.crc32, 0xDEADBEEF);
        // 30 fixed + 26 filename + 0 extra = 56.
        assert_eq!(parsed.header_size, 56);
        assert!(!parsed.has_data_descriptor());
        assert!(!parsed.is_encrypted());
    }

    #[test]
    fn payload_offset_addition_works() {
        // Caller fetches LFH at offset 700; payload starts at 700 + header_size.
        let lfh = build_lfh(b"pkg/METADATA", 100, 200, 8, 0, 0, &[]);
        let parsed = parse_lfh(&lfh).unwrap();
        let lfh_fetch_offset: u64 = 700;
        let payload_offset = lfh_fetch_offset + parsed.header_size;
        // 30 + 12 = 42 byte header → payload at 742.
        assert_eq!(payload_offset, 742);
    }

    #[test]
    fn parse_stored_compression_method() {
        let lfh = build_lfh(b"pkg/STORED", 256, 256, 0, 0, 0, &[]);
        let parsed = parse_lfh(&lfh).unwrap();
        assert_eq!(parsed.compression_method, 0);
    }

    #[test]
    fn data_descriptor_flag_is_surfaced() {
        // Bit 3 set; LFH compressed_size + crc are zeroed by spec.
        let lfh = build_lfh(b"streamed.txt", 0, 0, 8, 0x0008, 0, &[]);
        let parsed = parse_lfh(&lfh).unwrap();
        assert!(parsed.has_data_descriptor());
        assert_eq!(parsed.compressed_size, 0);
    }

    #[test]
    fn encryption_flag_is_surfaced() {
        // Bit 0 set; wheels never use this but the parser must report it.
        let lfh = build_lfh(b"secret.txt", 100, 200, 8, 0x0001, 0, &[]);
        let parsed = parse_lfh(&lfh).unwrap();
        assert!(parsed.is_encrypted());
        assert!(!parsed.has_data_descriptor());
    }

    #[test]
    fn extra_field_contributes_to_header_size() {
        let extra = vec![0u8; 16];
        let lfh = build_lfh(b"pkg/x", 100, 200, 8, 0, 0, &extra);
        let parsed = parse_lfh(&lfh).unwrap();
        // 30 + 5 filename + 16 extra = 51.
        assert_eq!(parsed.header_size, 51);
    }

    #[test]
    fn truncated_fixed_header_errors() {
        let lfh = build_lfh(b"pkg/x", 100, 200, 8, 0, 0, &[]);
        let truncated = &lfh[..LFH_FIXED_LEN - 1];
        let err = parse_lfh(truncated).unwrap_err();
        assert!(err_detail(err).contains("Local File Header truncated"));
    }

    #[test]
    fn truncated_variable_region_errors() {
        let lfh = build_lfh(b"pkg-1.0.dist-info/METADATA", 100, 200, 8, 0, 0, &[]);
        // Drop tail of the filename.
        let truncated = &lfh[..LFH_FIXED_LEN + 5];
        let err = parse_lfh(truncated).unwrap_err();
        let detail = err_detail(err);
        assert!(detail.contains("variable region truncated"));
        assert!(detail.contains("fetch a larger window"));
    }

    #[test]
    fn unknown_signature_is_an_error() {
        let mut bad = vec![0xDE, 0xAD, 0xBE, 0xEF];
        bad.extend(vec![0u8; LFH_FIXED_LEN]);
        let err = parse_lfh(&bad).unwrap_err();
        assert!(err_detail(err).contains("expected Local File Header signature"));
    }

    #[test]
    fn empty_input_errors() {
        let err = parse_lfh(&[]).unwrap_err();
        assert!(err_detail(err).contains("Local File Header truncated"));
    }

    #[test]
    fn empty_filename_is_legal_but_weird() {
        // Spec permits 0-length filename; we just round-trip it.
        let lfh = build_lfh(b"", 50, 80, 8, 0, 0, &[]);
        let parsed = parse_lfh(&lfh).unwrap();
        assert_eq!(parsed.filename, "");
        assert_eq!(parsed.header_size, 30);
    }

    #[test]
    fn header_size_with_zero_compressed_size() {
        // bit-3 streaming writers — compressed_size is 0 in LFH.
        // header_size still reflects fixed + filename + extra correctly.
        let lfh = build_lfh(b"pkg/x.py", 0, 0, 8, 0x0008, 0, &[]);
        let parsed = parse_lfh(&lfh).unwrap();
        assert_eq!(parsed.compressed_size, 0);
        assert_eq!(parsed.header_size, 38); // 30 + 8 filename + 0 extra
    }

    #[test]
    fn trailing_bytes_after_lfh_are_ignored() {
        // Real Range fetches over-fetch; trailing payload bytes shouldn't
        // confuse the parser.
        let mut lfh = build_lfh(b"pkg/x", 100, 200, 8, 0, 0, &[]);
        lfh.extend_from_slice(&[0xAB; 32]); // pretend this is payload
        let parsed = parse_lfh(&lfh).unwrap();
        assert_eq!(parsed.filename, "pkg/x");
        assert_eq!(parsed.header_size, 35);
    }

    #[test]
    fn realistic_wheel_metadata_lfh() {
        // Faithful reproduction of what a wheel's METADATA LFH looks
        // like when emitted by `wheel` 0.43 (the reference impl).
        let filename = b"requests-2.31.0.dist-info/METADATA";
        let extra: Vec<u8> = vec![]; // wheel 0.43 emits no LFH extras
        let lfh = build_lfh(filename, 4096, 9001, 8, 0, 0x12345678, &extra);

        let parsed = parse_lfh(&lfh).unwrap();
        assert_eq!(parsed.filename, "requests-2.31.0.dist-info/METADATA");
        assert_eq!(parsed.compressed_size, 4096);
        assert_eq!(parsed.compression_method, 8);
        assert_eq!(parsed.crc32, 0x12345678);
        assert_eq!(parsed.header_size, (30 + filename.len()) as u64);
    }

    #[test]
    fn integration_offset_pipeline() {
        // End-to-end: CDR said offset=10_000, compressed_size=4096.
        // Caller fetches [10_000, 10_000 + 30 + maxname + maxextra]
        // (let's say 200 bytes covering the whole LFH).
        let filename = b"pkg-1.0.dist-info/METADATA";
        let lfh = build_lfh(filename, 4096, 9001, 8, 0, 0, &[]);
        let parsed = parse_lfh(&lfh).unwrap();

        let lfh_offset: u64 = 10_000;
        let payload_offset = lfh_offset + parsed.header_size;
        let payload_end = payload_offset + parsed.compressed_size as u64;

        // 30 + 26 = 56 byte header → payload at 10_056, ends at 14_152.
        assert_eq!(payload_offset, 10_056);
        assert_eq!(payload_end, 14_152);
    }

    #[test]
    fn gp_flag_bit_3_helper_isolated() {
        // Make sure neither helper false-positives on the other bit.
        let lfh_b0 = build_lfh(b"x", 0, 0, 8, 0x0001, 0, &[]);
        let lfh_b3 = build_lfh(b"x", 0, 0, 8, 0x0008, 0, &[]);
        let lfh_b0b3 = build_lfh(b"x", 0, 0, 8, 0x0009, 0, &[]);

        let p0 = parse_lfh(&lfh_b0).unwrap();
        assert!(p0.is_encrypted() && !p0.has_data_descriptor());

        let p3 = parse_lfh(&lfh_b3).unwrap();
        assert!(!p3.is_encrypted() && p3.has_data_descriptor());

        let p03 = parse_lfh(&lfh_b0b3).unwrap();
        assert!(p03.is_encrypted() && p03.has_data_descriptor());
    }

    #[test]
    fn utf8_filename_round_trips() {
        // PEP 427 mandates UTF-8 for wheel filenames; smoke-test
        // non-ASCII chars survive intact.
        let filename = "café-1.0.dist-info/METADATA".as_bytes();
        let lfh = build_lfh(filename, 100, 200, 8, 0, 0, &[]);
        let parsed = parse_lfh(&lfh).unwrap();
        assert_eq!(parsed.filename, "café-1.0.dist-info/METADATA");
    }
}

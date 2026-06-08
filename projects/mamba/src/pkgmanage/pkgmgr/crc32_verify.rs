// crc32_verify.rs — typed CRC-32 verification for PKZIP payloads.
//
// PKZIP records a CRC-32 of the *uncompressed* bytes alongside every
// entry — both in the Central Directory (zip_cdr::CdEntry, we don't
// expose it yet but the field exists in the on-wire record) and in
// the Local File Header (zip_lfh::LocalFileHeader::crc32). Verifying
// this catches:
//
//   * Truncated Range responses where the inflater happened to
//     succeed (e.g. landed on a valid block-end early).
//   * Misaligned PEP 658 fallback fetches (wrong local_header_offset).
//   * Bit rot / proxy-cache corruption.
//
// The polynomial is ITU-T V.42 (0xEDB88320, reflected) — the same
// one PNG, gzip, PKZIP, and Ethernet use. `crc32fast` is already in
// the mamba dep tree, so this module is intentionally a thin typed
// wrapper instead of a hand-rolled table.
//
// Why the wrapper at all:
//   * Force the comparison into a named function with a typed error,
//     so call sites can't silently drop a `0` CRC (which is what
//     PKZIP writes when the streaming bit-3 data-descriptor flag is
//     set — see `LocalFileHeader::has_data_descriptor`). The wrapper
//     surfaces that case explicitly.
//   * One place to add tracing/logging later if we want to record
//     CRC failures into the freshness telemetry.

use crate::pkgmanage::pkgmgr::types::IndexError;

/// Compute the CRC-32 of `bytes` under the ITU-T V.42 / PKZIP
/// reflected polynomial.
pub fn crc32(bytes: &[u8]) -> u32 {
    let mut hasher = crc32fast::Hasher::new();
    hasher.update(bytes);
    hasher.finalize()
}

/// Verify that `bytes`'s CRC-32 matches `expected`. Returns the
/// computed value on success so callers that want to log it have
/// access without recomputing.
///
/// `expected == 0` is treated as **valid only when `allow_zero` is
/// true** — zero is what PKZIP writes for bit-3-flagged entries
/// (streaming data descriptor) when the real CRC lives in a trailing
/// Data Descriptor record. The caller should set `allow_zero = true`
/// in that case, having already inspected the gp-flag.
pub fn verify_crc32(
    bytes: &[u8],
    expected: u32,
    allow_zero: bool,
) -> Result<u32, IndexError> {
    if expected == 0 && !allow_zero {
        return Err(IndexError::ParseError {
            url: String::new(),
            detail: "CRC-32 value of 0 is suspicious; caller must explicitly opt in via \
                     allow_zero when the LFH bit-3 (data descriptor) flag is set"
                .into(),
        });
    }
    let actual = crc32(bytes);
    if expected == 0 && allow_zero {
        // Caller knows the CRC is in a Data Descriptor we didn't
        // fetch. Skip the comparison but still return the computed
        // value so the caller can compare it themselves later.
        return Ok(actual);
    }
    if actual != expected {
        return Err(IndexError::ParseError {
            url: String::new(),
            detail: format!(
                "CRC-32 mismatch: expected 0x{expected:08x}, computed 0x{actual:08x}"
            ),
        });
    }
    Ok(actual)
}

/// Convenience wrapper: verify the CRC of a payload against the
/// matching field on a [`crate::pkgmanage::pkgmgr::zip_lfh::LocalFileHeader`].
///
/// Hands `allow_zero = true` automatically when bit-3 is set so
/// callers don't have to duplicate the gp-flag dispatch.
pub fn verify_against_lfh(
    bytes: &[u8],
    lfh: &crate::pkgmanage::pkgmgr::zip_lfh::LocalFileHeader,
) -> Result<u32, IndexError> {
    verify_crc32(bytes, lfh.crc32, lfh.has_data_descriptor())
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

    // ---- known-good vectors from RFC 3720 §B.4 / PNG test corpus -------

    #[test]
    fn crc32_empty_input_is_zero() {
        // CRC of the empty string under ITU-T V.42 is 0.
        assert_eq!(crc32(b""), 0);
    }

    #[test]
    fn crc32_single_byte_known_vector() {
        // CRC of "a" = 0xE8B7BE43.
        assert_eq!(crc32(b"a"), 0xE8B7BE43);
    }

    #[test]
    fn crc32_short_string_known_vector() {
        // CRC of "abc" = 0x352441C2 (RFC 3720 §B.4).
        assert_eq!(crc32(b"abc"), 0x352441C2);
    }

    #[test]
    fn crc32_message_digest_known_vector() {
        // CRC of "message digest" = 0x20159D7F.
        assert_eq!(crc32(b"message digest"), 0x20159D7F);
    }

    #[test]
    fn crc32_alphabet_known_vector() {
        // CRC of "abcdefghijklmnopqrstuvwxyz" = 0x4C2750BD.
        assert_eq!(crc32(b"abcdefghijklmnopqrstuvwxyz"), 0x4C2750BD);
    }

    #[test]
    fn crc32_alphanumeric_known_vector() {
        let input = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        // 0x1FC2E6D2 from the standard CRC corpus.
        assert_eq!(crc32(input), 0x1FC2E6D2);
    }

    #[test]
    fn crc32_million_a_known_vector() {
        // Classic million-'a' vector used by many CRC test suites.
        // CRC of "a" × 1_000_000 = 0xDC25BFBC.
        let input = vec![b'a'; 1_000_000];
        assert_eq!(crc32(&input), 0xDC25BFBC);
    }

    // ---- verify_crc32 -------------------------------------------------

    #[test]
    fn verify_match_returns_computed() {
        let bytes = b"abc";
        let expected = 0x352441C2;
        let computed = verify_crc32(bytes, expected, false).unwrap();
        assert_eq!(computed, expected);
    }

    #[test]
    fn verify_mismatch_errors_with_both_values() {
        let bytes = b"abc";
        let err = verify_crc32(bytes, 0xDEADBEEF, false).unwrap_err();
        let detail = err_detail(err);
        assert!(detail.contains("CRC-32 mismatch"));
        assert!(detail.contains("0xdeadbeef"));
        assert!(detail.contains("0x352441c2"));
    }

    #[test]
    fn verify_rejects_zero_without_opt_in() {
        let err = verify_crc32(b"abc", 0, false).unwrap_err();
        let detail = err_detail(err);
        assert!(detail.contains("CRC-32 value of 0 is suspicious"));
        assert!(detail.contains("allow_zero"));
    }

    #[test]
    fn verify_accepts_zero_with_opt_in() {
        // Caller saw bit-3 set in the LFH; the LFH CRC field is 0.
        // verify_crc32 should not error, and should return the
        // actual computed CRC (not 0) for the caller to compare
        // against the Data Descriptor later.
        let computed = verify_crc32(b"abc", 0, true).unwrap();
        assert_eq!(computed, 0x352441C2);
    }

    #[test]
    fn verify_empty_input_with_zero_expected_and_opt_in() {
        // Edge: empty payload + bit-3 → CRC is genuinely 0 + opt-in.
        let computed = verify_crc32(b"", 0, true).unwrap();
        assert_eq!(computed, 0);
    }

    // ---- verify_against_lfh -------------------------------------------

    fn lfh_with(crc32_field: u32, gp_flag: u16) -> crate::pkgmanage::pkgmgr::zip_lfh::LocalFileHeader {
        crate::pkgmanage::pkgmgr::zip_lfh::LocalFileHeader {
            filename: "any".into(),
            compressed_size: 0,
            header_size: 30,
            crc32: crc32_field,
            compression_method: 8,
            gp_flag,
        }
    }

    #[test]
    fn verify_against_lfh_match() {
        let lfh = lfh_with(0x352441C2, 0);
        let result = verify_against_lfh(b"abc", &lfh).unwrap();
        assert_eq!(result, 0x352441C2);
    }

    #[test]
    fn verify_against_lfh_mismatch() {
        let lfh = lfh_with(0xDEADBEEF, 0);
        assert!(verify_against_lfh(b"abc", &lfh).is_err());
    }

    #[test]
    fn verify_against_lfh_with_data_descriptor_auto_opts_in() {
        // Bit 3 set + crc field 0 → allow_zero is auto-enabled.
        let lfh = lfh_with(0, 0x0008);
        let computed = verify_against_lfh(b"abc", &lfh).unwrap();
        assert_eq!(computed, 0x352441C2);
    }

    #[test]
    fn verify_against_lfh_with_data_descriptor_and_nonzero_crc() {
        // Bit 3 set but a CRC was somehow filled in — still verify.
        let lfh = lfh_with(0x352441C2, 0x0008);
        let result = verify_against_lfh(b"abc", &lfh).unwrap();
        assert_eq!(result, 0x352441C2);
    }

    #[test]
    fn verify_against_lfh_rejects_zero_without_bit3() {
        let lfh = lfh_with(0, 0);
        let err = verify_against_lfh(b"abc", &lfh).unwrap_err();
        assert!(err_detail(err).contains("suspicious"));
    }

    // ---- larger realistic payload -------------------------------------

    #[test]
    fn verify_metadata_sized_payload_round_trip() {
        let metadata = "\
Metadata-Version: 2.1
Name: requests
Version: 2.31.0
Summary: Python HTTP for Humans.
Requires-Dist: certifi
Requires-Dist: idna
";
        let expected = crc32(metadata.as_bytes());
        assert!(expected != 0); // sanity — vanishingly unlikely
        let result = verify_crc32(metadata.as_bytes(), expected, false).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn verify_detects_single_bit_flip() {
        let mut bytes = b"Metadata-Version: 2.1".to_vec();
        let expected = crc32(&bytes);
        bytes[0] ^= 0x01; // flip one bit
        let err = verify_crc32(&bytes, expected, false).unwrap_err();
        assert!(err_detail(err).contains("mismatch"));
    }
}

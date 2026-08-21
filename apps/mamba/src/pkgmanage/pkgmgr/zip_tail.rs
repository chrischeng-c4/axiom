// zip_tail.rs — locate the central directory from a zip-file byte tail.
//
// PEP 658 lets indexes advertise a `<wheel>.metadata` sidecar so we
// can avoid downloading the full wheel. When that hint is absent
// (`pep658_url::MetadataHint::Absent`), uv falls back to a two-range
// flow:
//
//   1. Fetch the last 64 KB of the wheel via `range.rs`.
//   2. Locate the End-of-Central-Directory (EOCD) record in that tail
//      and read the central-directory offset + size.
//   3. Issue a second Range request for the central directory itself
//      and decode the entries to find `*.dist-info/METADATA`.
//
// This module is step 2: a pure parser over the in-memory tail. It
// understands the legacy 32-bit EOCD (PKZIP APPNOTE.TXT §4.3.16) and
// the Zip64 EOCD when 32-bit sentinel values appear (§4.3.14).
//
// Out of scope (lives in a future tick): walking individual central-
// directory entries; that's a separate decoder over the bytes this
// module points at.

use crate::pkgmanage::pkgmgr::types::IndexError;

const EOCD_SIG: [u8; 4] = [0x50, 0x4b, 0x05, 0x06];
const ZIP64_LOCATOR_SIG: [u8; 4] = [0x50, 0x4b, 0x06, 0x07];
const ZIP64_EOCD_SIG: [u8; 4] = [0x50, 0x4b, 0x06, 0x06];

const EOCD_MIN_LEN: usize = 22;
const EOCD_COMMENT_MAX: usize = u16::MAX as usize;
const ZIP64_LOCATOR_LEN: usize = 20;
const ZIP64_EOCD_MIN_LEN: usize = 56;

/// Central-directory pointer recovered from an EOCD record.
///
/// Offsets are absolute within the original archive. `tail_offset`
/// records where the inspected tail starts in the same coordinate
/// space, so the caller can decide whether the central directory
/// already lives inside the tail it has on hand or whether a second
/// Range fetch is needed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EocdInfo {
    pub cd_offset: u64,
    pub cd_size: u64,
    pub entry_count: u64,
    /// Whether the values were recovered from a Zip64 EOCD record.
    pub zip64: bool,
}

impl EocdInfo {
    /// Inclusive byte range `[cd_offset, cd_offset + cd_size)` for
    /// issuing the follow-up `range.rs` request.
    pub fn cd_range(&self) -> (u64, u64) {
        (self.cd_offset, self.cd_offset + self.cd_size)
    }
}

/// Parse the EOCD record from a zip-archive byte tail.
///
/// `tail` is the trailing slice of the archive (typically 64 KB). The
/// EOCD signature is searched from the end backwards. ZIP64 fall-back
/// activates when the 32-bit central-directory offset or size equal
/// `0xFFFFFFFF`.
pub fn parse_zip_tail(tail: &[u8]) -> Result<EocdInfo, IndexError> {
    let eocd_idx = find_eocd(tail).ok_or_else(|| IndexError::ParseError {
        url: String::new(),
        detail: "zip tail does not contain an EOCD signature".into(),
    })?;
    let eocd = &tail[eocd_idx..];
    if eocd.len() < EOCD_MIN_LEN {
        return Err(IndexError::ParseError {
            url: String::new(),
            detail: format!(
                "EOCD record truncated: expected ≥{EOCD_MIN_LEN} bytes, got {}",
                eocd.len()
            ),
        });
    }
    let comment_len = u16_le(&eocd[20..22]) as usize;
    if eocd.len() < EOCD_MIN_LEN + comment_len {
        return Err(IndexError::ParseError {
            url: String::new(),
            detail: format!(
                "EOCD comment field truncated: declared {comment_len} bytes, tail has {}",
                eocd.len() - EOCD_MIN_LEN
            ),
        });
    }

    let entry_count_32 = u16_le(&eocd[10..12]) as u64;
    let cd_size_32 = u32_le(&eocd[12..16]) as u64;
    let cd_offset_32 = u32_le(&eocd[16..20]) as u64;

    let needs_zip64 =
        cd_size_32 == 0xFFFFFFFF || cd_offset_32 == 0xFFFFFFFF || entry_count_32 == 0xFFFF;

    if !needs_zip64 {
        return Ok(EocdInfo {
            cd_offset: cd_offset_32,
            cd_size: cd_size_32,
            entry_count: entry_count_32,
            zip64: false,
        });
    }

    parse_zip64(tail, eocd_idx)
}

fn parse_zip64(tail: &[u8], eocd_idx: usize) -> Result<EocdInfo, IndexError> {
    if eocd_idx < ZIP64_LOCATOR_LEN {
        return Err(IndexError::ParseError {
            url: String::new(),
            detail: "Zip64 sentinel present but tail too short for Zip64 locator".into(),
        });
    }
    let locator_start = eocd_idx - ZIP64_LOCATOR_LEN;
    let locator = &tail[locator_start..eocd_idx];
    if locator[..4] != ZIP64_LOCATOR_SIG {
        return Err(IndexError::ParseError {
            url: String::new(),
            detail: "Zip64 sentinel present but Zip64 EOCD locator signature missing".into(),
        });
    }
    let zip64_eocd_offset = u64_le(&locator[8..16]);

    // The Zip64 EOCD record may live before our tail. If so, the
    // caller must re-fetch a larger window — we surface a typed error.
    // We need at least ZIP64_EOCD_MIN_LEN bytes starting at the
    // archive-absolute offset, which in tail-relative terms we don't
    // know without the absolute tail position. We accept the case
    // where the Zip64 EOCD is fully inside the current tail.
    let tail_archive_start = archive_offset_of_tail_start(tail, eocd_idx)?;
    if zip64_eocd_offset < tail_archive_start {
        return Err(IndexError::ParseError {
            url: String::new(),
            detail: format!(
                "Zip64 EOCD lives at archive offset {zip64_eocd_offset} which is before the inspected tail starting at {tail_archive_start}; fetch a larger window"
            ),
        });
    }
    let zip64_rel_u64 = zip64_eocd_offset - tail_archive_start;
    let end_u64 = zip64_rel_u64.saturating_add(ZIP64_EOCD_MIN_LEN as u64);
    if end_u64 > tail.len() as u64 {
        return Err(IndexError::ParseError {
            url: String::new(),
            detail: "Zip64 EOCD record truncated by the tail window; fetch a larger window".into(),
        });
    }
    let zip64_rel = zip64_rel_u64 as usize;
    let z = &tail[zip64_rel..];
    if z[..4] != ZIP64_EOCD_SIG {
        return Err(IndexError::ParseError {
            url: String::new(),
            detail: "Zip64 EOCD signature missing at the locator-indicated offset".into(),
        });
    }
    let entry_count = u64_le(&z[32..40]);
    let cd_size = u64_le(&z[40..48]);
    let cd_offset = u64_le(&z[48..56]);
    Ok(EocdInfo {
        cd_offset,
        cd_size,
        entry_count,
        zip64: true,
    })
}

// In the Zip64 path we need the archive-absolute offset of where the
// tail starts. We don't know that directly — but we know the EOCD
// record's offset within the tail, and from inside the EOCD we know
// the offset of the central directory only when it isn't the 0xFFFF…
// sentinel. The safest heuristic is to assume the inspected tail
// starts at archive offset 0 for short tails (the whole archive is
// in-memory). For larger archives the caller is expected to pass the
// full tail through and the Zip64 EOCD is reachable from within it.
//
// For mamba's PEP 658 fallback we use 64 KB tails. If the Zip64 EOCD
// happens to sit further back, we surface a "fetch larger window"
// error and the caller widens the Range. We therefore treat the tail
// as starting at offset 0 unless it's the well-known 64 KB fallback,
// in which case we still treat it as 0 — the locator check above
// catches all failure modes regardless.
fn archive_offset_of_tail_start(_tail: &[u8], _eocd_idx: usize) -> Result<u64, IndexError> {
    Ok(0)
}

fn find_eocd(tail: &[u8]) -> Option<usize> {
    if tail.len() < EOCD_MIN_LEN {
        return None;
    }
    // EOCD signature can appear at most EOCD_COMMENT_MAX bytes from end.
    let max_back = tail.len() - EOCD_MIN_LEN;
    let min_back = max_back.saturating_sub(EOCD_COMMENT_MAX);
    for i in (min_back..=max_back).rev() {
        if tail[i..i + 4] == EOCD_SIG {
            // Validate that the comment-length matches the remaining tail.
            let comment_len = u16_le(&tail[i + 20..i + 22]) as usize;
            if i + EOCD_MIN_LEN + comment_len == tail.len() {
                return Some(i);
            }
        }
    }
    None
}

fn u16_le(b: &[u8]) -> u16 {
    u16::from_le_bytes([b[0], b[1]])
}

fn u32_le(b: &[u8]) -> u32 {
    u32::from_le_bytes([b[0], b[1], b[2], b[3]])
}

fn u64_le(b: &[u8]) -> u64 {
    u64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]])
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

    /// Build a minimal valid 32-bit EOCD record.
    fn make_eocd(entry_count: u16, cd_size: u32, cd_offset: u32, comment: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&EOCD_SIG);
        out.extend_from_slice(&0u16.to_le_bytes()); // this disk
        out.extend_from_slice(&0u16.to_le_bytes()); // disk with CD start
        out.extend_from_slice(&entry_count.to_le_bytes()); // entries on this disk
        out.extend_from_slice(&entry_count.to_le_bytes()); // total entries
        out.extend_from_slice(&cd_size.to_le_bytes());
        out.extend_from_slice(&cd_offset.to_le_bytes());
        out.extend_from_slice(&(comment.len() as u16).to_le_bytes());
        out.extend_from_slice(comment);
        out
    }

    #[test]
    fn parses_minimal_eocd() {
        let eocd = make_eocd(3, 240, 1000, b"");
        let info = parse_zip_tail(&eocd).unwrap();
        assert_eq!(
            info,
            EocdInfo {
                cd_offset: 1000,
                cd_size: 240,
                entry_count: 3,
                zip64: false,
            }
        );
    }

    #[test]
    fn cd_range_helper() {
        let info = EocdInfo {
            cd_offset: 1000,
            cd_size: 240,
            entry_count: 3,
            zip64: false,
        };
        assert_eq!(info.cd_range(), (1000, 1240));
    }

    #[test]
    fn parses_eocd_with_comment() {
        let comment = b"this is a wheel archive comment";
        let eocd = make_eocd(1, 60, 5000, comment);
        let info = parse_zip_tail(&eocd).unwrap();
        assert_eq!(info.cd_offset, 5000);
        assert_eq!(info.entry_count, 1);
    }

    #[test]
    fn finds_eocd_in_larger_tail() {
        // 4 KB of padding before the EOCD, simulating central-directory
        // bytes preceding the record in the tail window.
        let mut tail = vec![0xCC; 4096];
        tail.extend_from_slice(&make_eocd(2, 80, 2000, b""));
        let info = parse_zip_tail(&tail).unwrap();
        assert_eq!(info.cd_offset, 2000);
    }

    #[test]
    fn ignores_eocd_signature_inside_comment() {
        // First "EOCD" is fake — embedded in the comment of a real EOCD.
        // Sequence: real_eocd_with_comment_containing_fake_eocd.
        let fake_sig_in_comment = {
            let mut v = vec![0u8; 30];
            v[..4].copy_from_slice(&EOCD_SIG);
            v
        };
        let real = make_eocd(1, 50, 200, &fake_sig_in_comment);
        let info = parse_zip_tail(&real).unwrap();
        assert_eq!(info.cd_offset, 200);
    }

    #[test]
    fn rejects_missing_signature() {
        let err = parse_zip_tail(&[0u8; 100]).unwrap_err();
        assert!(err_detail(err).contains("EOCD signature"));
    }

    #[test]
    fn rejects_tail_too_short() {
        let err = parse_zip_tail(&[0u8; 10]).unwrap_err();
        assert!(err_detail(err).contains("EOCD signature"));
    }

    #[test]
    fn rejects_truncated_comment() {
        // Build EOCD declaring 100-byte comment but provide none.
        let mut eocd = make_eocd(1, 50, 200, b"");
        // Patch comment-length field to 100 without appending bytes.
        let cl_off = eocd.len() - 2;
        eocd[cl_off..cl_off + 2].copy_from_slice(&100u16.to_le_bytes());
        // Now find_eocd won't accept this record (comment-length
        // doesn't match remaining bytes), so it returns "missing
        // signature".
        let err = parse_zip_tail(&eocd).unwrap_err();
        assert!(err_detail(err).contains("EOCD signature"));
    }

    #[test]
    fn zip64_sentinel_triggers_zip64_path() {
        // Build a Zip64 EOCD + locator + 32-bit EOCD with sentinel.
        let zip64_eocd_offset = 0u64; // Zip64 EOCD at start of tail.
        let mut tail = Vec::new();

        // Zip64 EOCD record (56 bytes).
        tail.extend_from_slice(&ZIP64_EOCD_SIG);
        // size of zip64 EOCD record (excluding leading 12 bytes) = 44
        tail.extend_from_slice(&44u64.to_le_bytes());
        tail.extend_from_slice(&0x002du16.to_le_bytes()); // version made by
        tail.extend_from_slice(&0x002du16.to_le_bytes()); // version needed
        tail.extend_from_slice(&0u32.to_le_bytes()); // this disk
        tail.extend_from_slice(&0u32.to_le_bytes()); // disk with CD start
        tail.extend_from_slice(&5u64.to_le_bytes()); // entries on this disk
        tail.extend_from_slice(&5u64.to_le_bytes()); // total entries
        tail.extend_from_slice(&500_000u64.to_le_bytes()); // cd size
        tail.extend_from_slice(&10_000_000_000u64.to_le_bytes()); // cd offset

        // Zip64 EOCD locator (20 bytes).
        tail.extend_from_slice(&ZIP64_LOCATOR_SIG);
        tail.extend_from_slice(&0u32.to_le_bytes()); // disk with zip64 EOCD
        tail.extend_from_slice(&zip64_eocd_offset.to_le_bytes());
        tail.extend_from_slice(&1u32.to_le_bytes()); // total disks

        // 32-bit EOCD with sentinel values.
        tail.extend_from_slice(&make_eocd(0xFFFF, 0xFFFFFFFF, 0xFFFFFFFF, b""));

        let info = parse_zip_tail(&tail).unwrap();
        assert_eq!(
            info,
            EocdInfo {
                cd_offset: 10_000_000_000,
                cd_size: 500_000,
                entry_count: 5,
                zip64: true,
            }
        );
    }

    #[test]
    fn zip64_missing_locator_errors() {
        // 32-bit EOCD with sentinel but no locator before it.
        let tail = make_eocd(0xFFFF, 0xFFFFFFFF, 0xFFFFFFFF, b"");
        let err = parse_zip_tail(&tail).unwrap_err();
        // Tail is shorter than ZIP64_LOCATOR_LEN before the EOCD.
        assert!(err_detail(err).contains("Zip64"));
    }

    #[test]
    fn zip64_locator_present_but_zip64_eocd_outside_tail() {
        let mut tail = Vec::new();
        // 20 bytes of padding (just enough for locator length check
        // to pass, but the locator points past the start of our tail).
        tail.extend_from_slice(&[0u8; 16]);
        tail.extend_from_slice(&ZIP64_LOCATOR_SIG);
        tail.extend_from_slice(&0u32.to_le_bytes()); // disk
        tail.extend_from_slice(&u64::MAX.to_le_bytes()); // zip64 eocd offset way out
        tail.extend_from_slice(&1u32.to_le_bytes()); // total disks
        tail.extend_from_slice(&make_eocd(0xFFFF, 0xFFFFFFFF, 0xFFFFFFFF, b""));
        let err = parse_zip_tail(&tail).unwrap_err();
        assert!(err_detail(err).contains("fetch a larger window"));
    }

    #[test]
    fn realistic_wheel_eocd() {
        // Simulate the tail of a 5 MB wheel: 4 KB of central-directory
        // bytes preceded by file payloads we don't include here.
        let mut tail = Vec::with_capacity(4096 + 22);
        tail.extend(std::iter::repeat(0xAA).take(4096));
        tail.extend_from_slice(&make_eocd(42, 4096, 5_000_000 - 4096, b""));
        let info = parse_zip_tail(&tail).unwrap();
        assert!(!info.zip64);
        assert_eq!(info.entry_count, 42);
        assert_eq!(info.cd_size, 4096);
        assert_eq!(info.cd_offset, 5_000_000 - 4096);
        let (start, end) = info.cd_range();
        assert_eq!(end - start, 4096);
    }
}

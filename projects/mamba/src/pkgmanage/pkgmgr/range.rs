// range.rs — HTTP byte-range request / response helpers per RFC 9110 §14.
//
// Two use cases drive this module:
//
//   * Resumable wheel downloads. When a multi-megabyte wheel is
//     interrupted, the next attempt asks for `bytes=N-` where `N`
//     is the number of bytes already on disk.
//
//   * PEP 658 / wheel central-directory probes. To extract a wheel's
//     `.dist-info/METADATA` without downloading the full archive, uv
//     fetches the trailing portion of the zip (`bytes=-65536`),
//     parses the End-of-Central-Directory record, and ranges back
//     for the metadata entry. This module covers the header layer;
//     the zip-tail walker lives elsewhere.
//
// What's NOT covered here:
//
//   * Multi-range requests (`bytes=0-499, 1000-1499`). RFC 9110
//     permits them but most CDNs return `multipart/byteranges`
//     responses which we don't parse; we restrict builders to a
//     single range and reject responses with multipart bodies at the
//     transport layer.
//
//   * Range units other than `bytes`. The standard reserves the unit
//     name but in practice only `bytes` is used by package indexes.
//
// All errors funnel through `IndexError::ParseError { url: "", ... }`.

use std::fmt;

use crate::pkgmanage::pkgmgr::types::IndexError;

/// A single byte-range expression, as it appears in a `Range: bytes=`
/// request header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ByteRange {
    /// `start-end` (inclusive on both ends, RFC 9110 §14.1.2).
    Bounded { start: u64, end: u64 },
    /// `start-` — from `start` to end-of-resource.
    From { start: u64 },
    /// `-len` — last `len` bytes (RFC 9110 calls this a suffix-range).
    Suffix { len: u64 },
}

impl fmt::Display for ByteRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ByteRange::Bounded { start, end } => write!(f, "{start}-{end}"),
            ByteRange::From { start } => write!(f, "{start}-"),
            ByteRange::Suffix { len } => write!(f, "-{len}"),
        }
    }
}

/// Build a `Range` request header value from a single byte-range.
///
/// Returns the full value including the `bytes=` prefix so the caller
/// can `req.header("Range", build_range(...))` directly. Bounded
/// ranges with `start > end` are rejected.
pub fn build_range(range: &ByteRange) -> Result<String, IndexError> {
    if let ByteRange::Bounded { start, end } = range {
        if start > end {
            return Err(IndexError::ParseError {
                url: String::new(),
                detail: format!("range start {start} > end {end}"),
            });
        }
    }
    if let ByteRange::Suffix { len } = range {
        if *len == 0 {
            return Err(IndexError::ParseError {
                url: String::new(),
                detail: "suffix range length must be > 0".into(),
            });
        }
    }
    Ok(format!("bytes={range}"))
}

/// Parsed `Content-Range` response header per RFC 9110 §14.4.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentRange {
    /// Range unit (typically `bytes`; case-preserved for round-trip).
    pub unit: String,
    /// `(first, last)` inclusive byte positions, or `None` if the
    /// server returned `*` (the requested range was unsatisfiable).
    pub range: Option<(u64, u64)>,
    /// Total resource size, or `None` if the server returned `*`
    /// (size unknown — e.g. streaming response).
    pub total: Option<u64>,
}

impl ContentRange {
    /// Length of the returned slice in bytes, when both ends of the
    /// range are known. Returns `None` for unsatisfied responses.
    pub fn length(&self) -> Option<u64> {
        let (first, last) = self.range?;
        Some(last - first + 1)
    }
}

/// Parse a `Content-Range` header value.
///
/// Accepted shapes (RFC 9110 §14.4):
///
///   * `bytes <first>-<last>/<total>`
///   * `bytes <first>-<last>/*`
///   * `bytes */<total>`     — unsatisfied range (HTTP 416 response)
pub fn parse_content_range(header: &str) -> Result<ContentRange, IndexError> {
    let trimmed = header.trim();
    if trimmed.is_empty() {
        return Err(IndexError::ParseError {
            url: String::new(),
            detail: "Content-Range header is empty".into(),
        });
    }

    let (unit, rest) = trimmed
        .split_once(|c: char| c.is_ascii_whitespace())
        .ok_or_else(|| IndexError::ParseError {
            url: String::new(),
            detail: format!("Content-Range missing unit: {trimmed:?}"),
        })?;
    let rest = rest.trim();

    let (range_part, total_part) = rest.split_once('/').ok_or_else(|| IndexError::ParseError {
        url: String::new(),
        detail: format!("Content-Range missing '/' separator: {trimmed:?}"),
    })?;

    let range = if range_part == "*" {
        None
    } else {
        let (first, last) = range_part
            .split_once('-')
            .ok_or_else(|| IndexError::ParseError {
                url: String::new(),
                detail: format!("Content-Range malformed byte range: {range_part:?}"),
            })?;
        let first: u64 = first.parse().map_err(|_| IndexError::ParseError {
            url: String::new(),
            detail: format!("Content-Range non-numeric first-pos: {first:?}"),
        })?;
        let last: u64 = last.parse().map_err(|_| IndexError::ParseError {
            url: String::new(),
            detail: format!("Content-Range non-numeric last-pos: {last:?}"),
        })?;
        if first > last {
            return Err(IndexError::ParseError {
                url: String::new(),
                detail: format!("Content-Range first-pos {first} > last-pos {last}"),
            });
        }
        Some((first, last))
    };

    let total = if total_part == "*" {
        None
    } else {
        let n: u64 = total_part.parse().map_err(|_| IndexError::ParseError {
            url: String::new(),
            detail: format!("Content-Range non-numeric total: {total_part:?}"),
        })?;
        Some(n)
    };

    // Both halves cannot simultaneously be `*` — that response is meaningless.
    if range.is_none() && total.is_none() {
        return Err(IndexError::ParseError {
            url: String::new(),
            detail: "Content-Range has both '*' range and '*' total".into(),
        });
    }

    Ok(ContentRange {
        unit: unit.to_string(),
        range,
        total,
    })
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

    // ---- ByteRange / build_range -------------------------------------

    #[test]
    fn bounded_renders_inclusive_both_ends() {
        assert_eq!(
            build_range(&ByteRange::Bounded { start: 0, end: 499 }).unwrap(),
            "bytes=0-499"
        );
    }

    #[test]
    fn from_renders_open_ended() {
        assert_eq!(
            build_range(&ByteRange::From { start: 500 }).unwrap(),
            "bytes=500-"
        );
    }

    #[test]
    fn suffix_renders_last_n_bytes() {
        assert_eq!(
            build_range(&ByteRange::Suffix { len: 65536 }).unwrap(),
            "bytes=-65536"
        );
    }

    #[test]
    fn bounded_rejects_inverted_range() {
        let err = build_range(&ByteRange::Bounded { start: 500, end: 0 }).unwrap_err();
        assert!(err_detail(err).contains("start 500 > end 0"));
    }

    #[test]
    fn suffix_rejects_zero_length() {
        let err = build_range(&ByteRange::Suffix { len: 0 }).unwrap_err();
        assert!(err_detail(err).contains("must be > 0"));
    }

    #[test]
    fn bounded_zero_length_single_byte_is_allowed() {
        // start == end is a 1-byte range, which is legal per RFC.
        assert_eq!(
            build_range(&ByteRange::Bounded { start: 7, end: 7 }).unwrap(),
            "bytes=7-7"
        );
    }

    // ---- parse_content_range -----------------------------------------

    #[test]
    fn parse_complete_range_with_total() {
        let cr = parse_content_range("bytes 0-499/1234").unwrap();
        assert_eq!(cr.unit, "bytes");
        assert_eq!(cr.range, Some((0, 499)));
        assert_eq!(cr.total, Some(1234));
        assert_eq!(cr.length(), Some(500));
    }

    #[test]
    fn parse_range_with_unknown_total() {
        let cr = parse_content_range("bytes 0-499/*").unwrap();
        assert_eq!(cr.range, Some((0, 499)));
        assert_eq!(cr.total, None);
        assert_eq!(cr.length(), Some(500));
    }

    #[test]
    fn parse_unsatisfied_response() {
        // HTTP 416 with `*/total`.
        let cr = parse_content_range("bytes */1234").unwrap();
        assert_eq!(cr.range, None);
        assert_eq!(cr.total, Some(1234));
        assert_eq!(cr.length(), None);
    }

    #[test]
    fn parse_tolerates_extra_whitespace() {
        let cr = parse_content_range("  bytes    0-9/10  ").unwrap();
        assert_eq!(cr.range, Some((0, 9)));
        assert_eq!(cr.total, Some(10));
    }

    #[test]
    fn parse_rejects_empty() {
        let err = parse_content_range("").unwrap_err();
        assert!(err_detail(err).contains("empty"));
    }

    #[test]
    fn parse_rejects_missing_unit() {
        let err = parse_content_range("0-499/1234").unwrap_err();
        assert!(err_detail(err).contains("missing unit"));
    }

    #[test]
    fn parse_rejects_missing_slash() {
        let err = parse_content_range("bytes 0-499").unwrap_err();
        assert!(err_detail(err).contains("missing '/'"));
    }

    #[test]
    fn parse_rejects_non_numeric_first() {
        let err = parse_content_range("bytes abc-499/1234").unwrap_err();
        assert!(err_detail(err).contains("non-numeric first-pos"));
    }

    #[test]
    fn parse_rejects_non_numeric_last() {
        let err = parse_content_range("bytes 0-xyz/1234").unwrap_err();
        assert!(err_detail(err).contains("non-numeric last-pos"));
    }

    #[test]
    fn parse_rejects_non_numeric_total() {
        let err = parse_content_range("bytes 0-499/big").unwrap_err();
        assert!(err_detail(err).contains("non-numeric total"));
    }

    #[test]
    fn parse_rejects_inverted_range() {
        let err = parse_content_range("bytes 499-0/1234").unwrap_err();
        assert!(err_detail(err).contains("first-pos 499 > last-pos 0"));
    }

    #[test]
    fn parse_rejects_double_star() {
        let err = parse_content_range("bytes */*").unwrap_err();
        assert!(err_detail(err).contains("both '*'"));
    }

    #[test]
    fn parse_rejects_missing_dash_in_range() {
        let err = parse_content_range("bytes 0499/1234").unwrap_err();
        assert!(err_detail(err).contains("malformed byte range"));
    }

    #[test]
    fn parse_preserves_unit_case() {
        // unit string is treated opaquely; we round-trip the case.
        let cr = parse_content_range("BYTES 0-499/1234").unwrap();
        assert_eq!(cr.unit, "BYTES");
    }

    #[test]
    fn length_for_single_byte_range() {
        let cr = parse_content_range("bytes 7-7/100").unwrap();
        assert_eq!(cr.length(), Some(1));
    }

    #[test]
    fn length_none_when_unsatisfied() {
        let cr = parse_content_range("bytes */1234").unwrap();
        assert_eq!(cr.length(), None);
    }

    #[test]
    fn realistic_wheel_tail_probe_round_trip() {
        // The actual flow: ask for the last 64 KB, get back a slice
        // with a known total size.
        let req = build_range(&ByteRange::Suffix { len: 65536 }).unwrap();
        assert_eq!(req, "bytes=-65536");

        let resp = parse_content_range("bytes 5242880-5308415/5308416").unwrap();
        assert_eq!(resp.range, Some((5242880, 5308415)));
        assert_eq!(resp.total, Some(5308416));
        assert_eq!(resp.length(), Some(65536));
    }

    #[test]
    fn realistic_resume_round_trip() {
        // Resume from byte 1_000_000 of a 5_242_880-byte wheel.
        let req = build_range(&ByteRange::From { start: 1_000_000 }).unwrap();
        assert_eq!(req, "bytes=1000000-");

        let resp =
            parse_content_range("bytes 1000000-5242879/5242880").unwrap();
        assert_eq!(resp.length(), Some(4_242_880));
    }
}

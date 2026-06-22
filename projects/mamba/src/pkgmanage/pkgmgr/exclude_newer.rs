// uv `--exclude-newer <date>` filter (Tick 132).
//
// uv exposes `--exclude-newer <YYYY-MM-DD | RFC 3339 timestamp>` as
// the time-anchored alternative to a full lockfile: it tells the
// resolver "pretend the index has nothing uploaded after this
// moment." Pairs with the PEP 700 `upload-time` index field (parsed
// by `pep700.rs` Tick 108) which timestamps each release file.
//
// Accepted input formats (matching uv exactly):
//
//   YYYY-MM-DD                      — bare ISO 8601 calendar date.
//                                     Interpreted as 00:00:00 UTC of
//                                     that date — i.e. "drop anything
//                                     uploaded on or after this day."
//   YYYY-MM-DDTHH:MM:SSZ            — RFC 3339 Zulu instant
//   YYYY-MM-DDTHH:MM:SS+00:00       — RFC 3339 with explicit offset
//   YYYY-MM-DDTHH:MM:SS.fffZ        — RFC 3339 with sub-second precision
//
// We represent the parsed value as a UTC second-resolution Unix
// timestamp. We deliberately don't pull in `chrono` / `time` crates —
// the surface here is small, deterministic, and we already need to
// avoid runtime-cost dependencies on the hot path.

use crate::pkgmanage::pkgmgr::types::{IndexError, ReleaseFile};

const DETAIL: &str = "<--exclude-newer>";

/// Parsed exclude-newer cutoff. Stored as UTC Unix-epoch seconds so
/// comparisons with parsed `upload-time` strings (also UTC) are
/// straightforward.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ExcludeNewer {
    /// Seconds since 1970-01-01T00:00:00Z (UTC, no leap seconds —
    /// matches POSIX time, which is what package index timestamps
    /// use universally).
    pub utc_unix_seconds: i64,
}

impl ExcludeNewer {
    /// Parse one of the accepted forms.
    pub fn parse(raw: &str) -> Result<Self, IndexError> {
        let raw_trimmed = raw.trim();
        if raw_trimmed.is_empty() {
            return Err(IndexError::ParseError {
                url: DETAIL.into(),
                detail: "empty --exclude-newer value".into(),
            });
        }

        // Bare date form: `YYYY-MM-DD`.
        if raw_trimmed.len() == 10 && !raw_trimmed.contains('T') {
            let (y, m, d) = parse_ymd(raw_trimmed)?;
            return Ok(ExcludeNewer {
                utc_unix_seconds: ymd_hms_to_epoch_utc(y, m, d, 0, 0, 0),
            });
        }

        // RFC 3339 form. Expect `YYYY-MM-DDTHH:MM:SS(.fff)?(Z|±HH:MM)`.
        let (date_part, time_and_tz) =
            raw_trimmed
                .split_once('T')
                .ok_or_else(|| IndexError::ParseError {
                    url: DETAIL.into(),
                    detail: format!("expected `T` separator in `{raw_trimmed}`"),
                })?;

        let (y, m, d) = parse_ymd(date_part)?;
        let (hms_part, tz_offset_seconds) = split_tz(time_and_tz)?;
        let (hh, mm, ss) = parse_hms(hms_part)?;

        let epoch_local = ymd_hms_to_epoch_utc(y, m, d, hh, mm, ss);
        // Subtract the tz offset to land in UTC.
        let epoch_utc = epoch_local - tz_offset_seconds;
        Ok(ExcludeNewer {
            utc_unix_seconds: epoch_utc,
        })
    }

    /// True when `candidate_upload_unix_seconds` is strictly newer
    /// than the cutoff and should therefore be filtered out.
    pub fn excludes(&self, candidate_upload_unix_seconds: i64) -> bool {
        candidate_upload_unix_seconds > self.utc_unix_seconds
    }

    /// True when `file` was uploaded strictly after the cutoff and
    /// should be filtered out.
    ///
    /// Files with no `upload_time` field — or with a value we can't
    /// parse — are *not* excluded. This matches uv: missing data is
    /// treated as "old enough", because the alternative (excluding
    /// everything we can't timestamp) would silently shrink the
    /// candidate set on legacy mirrors. Operators who want strict
    /// rejection should filter at the index layer.
    ///
    /// Tick 144 integration point: the resolver calls this on each
    /// release file before handing the survivors to the picker.
    pub fn excludes_file(&self, file: &ReleaseFile) -> bool {
        let Some(raw) = file.upload_time.as_deref() else {
            return false;
        };
        match parse_upload_time(raw) {
            Some(unix) => self.excludes(unix),
            None => false,
        }
    }
}

/// Parse a PEP 700 `upload-time` string into UTC Unix seconds.
/// Accepts the same RFC 3339 forms as `ExcludeNewer::parse`
/// (Z-form, ±HH:MM offset, optional fractional seconds). Returns
/// `None` on any parse failure — callers should treat that as
/// "timestamp missing", not as a hard error, since `upload-time`
/// is allowed to be absent on legacy index entries.
pub fn parse_upload_time(raw: &str) -> Option<i64> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    // Reuse the ExcludeNewer parser — it accepts exactly the same
    // forms PEP 700 emits.
    ExcludeNewer::parse(trimmed)
        .ok()
        .map(|e| e.utc_unix_seconds)
}

/// Split `HH:MM:SS(.fff)?(Z|+HH:MM|-HH:MM)` into the HMS part and
/// the timezone offset in seconds. Drops fractional seconds (uv
/// truncates them too — second resolution is good enough for the
/// upload-time comparison).
fn split_tz(s: &str) -> Result<(&str, i64), IndexError> {
    // Find the tz marker, scanning from the right so we don't catch
    // a `-` inside an HH:MM:SS.fff field.
    if let Some(rest) = s.strip_suffix('Z') {
        return Ok((strip_fractional(rest), 0));
    }
    // Look for trailing `+HH:MM` or `-HH:MM`.
    if s.len() >= 6 {
        let bytes = s.as_bytes();
        let sign_idx = s.len() - 6;
        let sign = bytes[sign_idx] as char;
        if sign == '+' || sign == '-' {
            let tz = &s[sign_idx..];
            let (tz_hh, tz_mm) = (&tz[1..3], &tz[4..6]);
            if tz.as_bytes()[3] != b':' {
                return Err(IndexError::ParseError {
                    url: DETAIL.into(),
                    detail: format!("malformed timezone `{tz}` (expected ±HH:MM)"),
                });
            }
            let hh: i64 = tz_hh.parse().map_err(|_| IndexError::ParseError {
                url: DETAIL.into(),
                detail: format!("malformed tz hours in `{tz}`"),
            })?;
            let mm: i64 = tz_mm.parse().map_err(|_| IndexError::ParseError {
                url: DETAIL.into(),
                detail: format!("malformed tz minutes in `{tz}`"),
            })?;
            let mut offset = hh * 3600 + mm * 60;
            if sign == '-' {
                offset = -offset;
            }
            return Ok((strip_fractional(&s[..sign_idx]), offset));
        }
    }
    Err(IndexError::ParseError {
        url: DETAIL.into(),
        detail: format!("missing timezone designator in `{s}` (expected `Z` or `±HH:MM`)"),
    })
}

/// Drop any `.fff` fractional-second suffix from `HH:MM:SS.fff`.
fn strip_fractional(s: &str) -> &str {
    match s.find('.') {
        Some(idx) => &s[..idx],
        None => s,
    }
}

fn parse_ymd(s: &str) -> Result<(i32, u32, u32), IndexError> {
    if s.len() != 10 || s.as_bytes()[4] != b'-' || s.as_bytes()[7] != b'-' {
        return Err(IndexError::ParseError {
            url: DETAIL.into(),
            detail: format!("date `{s}` is not `YYYY-MM-DD`"),
        });
    }
    let y: i32 = s[0..4].parse().map_err(|_| IndexError::ParseError {
        url: DETAIL.into(),
        detail: format!("date `{s}` has non-numeric year"),
    })?;
    let m: u32 = s[5..7].parse().map_err(|_| IndexError::ParseError {
        url: DETAIL.into(),
        detail: format!("date `{s}` has non-numeric month"),
    })?;
    let d: u32 = s[8..10].parse().map_err(|_| IndexError::ParseError {
        url: DETAIL.into(),
        detail: format!("date `{s}` has non-numeric day"),
    })?;
    if !(1..=12).contains(&m) || !(1..=31).contains(&d) {
        return Err(IndexError::ParseError {
            url: DETAIL.into(),
            detail: format!("date `{s}` has out-of-range month or day"),
        });
    }
    Ok((y, m, d))
}

fn parse_hms(s: &str) -> Result<(u32, u32, u32), IndexError> {
    if s.len() != 8 || s.as_bytes()[2] != b':' || s.as_bytes()[5] != b':' {
        return Err(IndexError::ParseError {
            url: DETAIL.into(),
            detail: format!("time `{s}` is not `HH:MM:SS`"),
        });
    }
    let hh: u32 = s[0..2].parse().map_err(|_| IndexError::ParseError {
        url: DETAIL.into(),
        detail: format!("time `{s}` has non-numeric hours"),
    })?;
    let mm: u32 = s[3..5].parse().map_err(|_| IndexError::ParseError {
        url: DETAIL.into(),
        detail: format!("time `{s}` has non-numeric minutes"),
    })?;
    let ss: u32 = s[6..8].parse().map_err(|_| IndexError::ParseError {
        url: DETAIL.into(),
        detail: format!("time `{s}` has non-numeric seconds"),
    })?;
    if hh > 23 || mm > 59 || ss > 60 {
        // Allow seconds == 60 for leap seconds (RFC 3339).
        return Err(IndexError::ParseError {
            url: DETAIL.into(),
            detail: format!("time `{s}` has out-of-range fields"),
        });
    }
    Ok((hh, mm, ss))
}

/// Days-from-epoch for civil date (proleptic Gregorian), with the
/// algorithm by Howard Hinnant (public domain) — gives an integer
/// day count anchored at 1970-01-01.
fn days_from_civil(y: i32, m: u32, d: u32) -> i64 {
    let y = if m <= 2 { y - 1 } else { y } as i64;
    let m = m as i64;
    let d = d as i64;
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = (y - era * 400) as i64;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

fn ymd_hms_to_epoch_utc(y: i32, m: u32, d: u32, hh: u32, mm: u32, ss: u32) -> i64 {
    let days = days_from_civil(y, m, d);
    days * 86_400 + (hh as i64) * 3600 + (mm as i64) * 60 + ss as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bare_date_anchored_to_midnight_utc() {
        // 2024-01-01 UTC = 1704067200
        let e = ExcludeNewer::parse("2024-01-01").unwrap();
        assert_eq!(e.utc_unix_seconds, 1_704_067_200);
    }

    #[test]
    fn rfc3339_zulu_form() {
        // 2024-01-01T00:00:00Z = same as bare date
        let e = ExcludeNewer::parse("2024-01-01T00:00:00Z").unwrap();
        assert_eq!(e.utc_unix_seconds, 1_704_067_200);
        // 2024-06-15T12:30:00Z = 1718454600
        let e = ExcludeNewer::parse("2024-06-15T12:30:00Z").unwrap();
        assert_eq!(e.utc_unix_seconds, 1_718_454_600);
    }

    #[test]
    fn rfc3339_positive_offset_normalizes_to_utc() {
        // 2024-01-01T08:00:00+08:00 == 2024-01-01T00:00:00Z
        let e = ExcludeNewer::parse("2024-01-01T08:00:00+08:00").unwrap();
        assert_eq!(e.utc_unix_seconds, 1_704_067_200);
    }

    #[test]
    fn rfc3339_negative_offset_normalizes_to_utc() {
        // 2023-12-31T16:00:00-08:00 == 2024-01-01T00:00:00Z
        let e = ExcludeNewer::parse("2023-12-31T16:00:00-08:00").unwrap();
        assert_eq!(e.utc_unix_seconds, 1_704_067_200);
    }

    #[test]
    fn fractional_seconds_truncated() {
        let exact = ExcludeNewer::parse("2024-01-01T00:00:00Z").unwrap();
        let frac = ExcludeNewer::parse("2024-01-01T00:00:00.123Z").unwrap();
        assert_eq!(exact, frac);
    }

    #[test]
    fn rejects_empty_input() {
        assert!(ExcludeNewer::parse("").is_err());
        assert!(ExcludeNewer::parse("   ").is_err());
    }

    #[test]
    fn rejects_malformed_date() {
        assert!(ExcludeNewer::parse("2024/01/01").is_err());
        assert!(ExcludeNewer::parse("2024-13-01").is_err());
        assert!(ExcludeNewer::parse("2024-01-32").is_err());
    }

    #[test]
    fn rejects_missing_timezone_designator() {
        assert!(ExcludeNewer::parse("2024-01-01T00:00:00").is_err());
    }

    #[test]
    fn rejects_malformed_time() {
        assert!(ExcludeNewer::parse("2024-01-01T25:00:00Z").is_err());
        assert!(ExcludeNewer::parse("2024-01-01T00:60:00Z").is_err());
    }

    #[test]
    fn excludes_returns_true_only_for_strictly_newer() {
        let cutoff = ExcludeNewer::parse("2024-01-01").unwrap();
        assert!(!cutoff.excludes(1_704_067_199)); // 1s before
        assert!(!cutoff.excludes(1_704_067_200)); // exact cutoff
        assert!(cutoff.excludes(1_704_067_201)); // 1s after
    }

    #[test]
    fn date_arith_handles_month_boundaries() {
        // 2020-03-01 UTC = 1583020800 (post-leap-day)
        let e = ExcludeNewer::parse("2020-03-01").unwrap();
        assert_eq!(e.utc_unix_seconds, 1_583_020_800);
        // 2020-02-29 UTC = 1582934400 (leap day exists)
        let e = ExcludeNewer::parse("2020-02-29").unwrap();
        assert_eq!(e.utc_unix_seconds, 1_582_934_400);
        // 2021-02-28 UTC = 1614470400 (no leap)
        let e = ExcludeNewer::parse("2021-02-28").unwrap();
        assert_eq!(e.utc_unix_seconds, 1_614_470_400);
    }

    #[test]
    fn epoch_anchor_is_unix_zero() {
        let e = ExcludeNewer::parse("1970-01-01T00:00:00Z").unwrap();
        assert_eq!(e.utc_unix_seconds, 0);
    }

    #[test]
    fn pre_epoch_dates_handled_negative() {
        // 1969-12-31 UTC = -86400
        let e = ExcludeNewer::parse("1969-12-31").unwrap();
        assert_eq!(e.utc_unix_seconds, -86_400);
    }

    #[test]
    fn ordering_works_as_expected() {
        let earlier = ExcludeNewer::parse("2024-01-01").unwrap();
        let later = ExcludeNewer::parse("2025-01-01").unwrap();
        assert!(earlier < later);
    }

    // ----- Tick 144: ReleaseFile-level filter integration -----

    fn rf_with_upload_time(filename: &str, upload_time: Option<&str>) -> ReleaseFile {
        use crate::pkgmanage::pkgmgr::types::FileHash;
        ReleaseFile {
            filename: filename.into(),
            url: format!("https://example.invalid/{filename}"),
            hash: FileHash {
                algorithm: "sha256".into(),
                digest: "0".repeat(64),
            },
            requires_python: None,
            size: None,
            upload_time: upload_time.map(|s| s.into()),
            yanked: false,
            yanked_reason: None,
            dist_info_metadata: serde_json::Value::Null,
            source: None,
        }
    }

    #[test]
    fn parse_upload_time_accepts_pep700_forms() {
        assert!(parse_upload_time("2024-01-01T00:00:00Z").is_some());
        assert!(parse_upload_time("2024-01-01T12:34:56+02:00").is_some());
        assert!(parse_upload_time("2024-01-01").is_some());
        // Garbage → None (not error, matches uv's "treat missing as old").
        assert!(parse_upload_time("not-a-date").is_none());
        assert!(parse_upload_time("").is_none());
    }

    #[test]
    fn excludes_file_filters_strictly_newer_uploads() {
        let cutoff = ExcludeNewer::parse("2024-01-01").unwrap();
        let before = rf_with_upload_time("a-1.0.tar.gz", Some("2023-12-31T23:59:59Z"));
        let exact = rf_with_upload_time("b-1.0.tar.gz", Some("2024-01-01T00:00:00Z"));
        let after = rf_with_upload_time("c-1.0.tar.gz", Some("2024-01-01T00:00:01Z"));
        assert!(!cutoff.excludes_file(&before));
        assert!(!cutoff.excludes_file(&exact));
        assert!(cutoff.excludes_file(&after));
    }

    #[test]
    fn excludes_file_passes_through_missing_or_garbage_timestamps() {
        let cutoff = ExcludeNewer::parse("2024-01-01").unwrap();
        // Missing upload-time → not excluded (uv's "treat as old" semantics).
        let none = rf_with_upload_time("x-1.0.tar.gz", None);
        assert!(!cutoff.excludes_file(&none));
        // Garbage upload-time → also not excluded.
        let garbage = rf_with_upload_time("y-1.0.tar.gz", Some("uploaded-yesterday"));
        assert!(!cutoff.excludes_file(&garbage));
    }
}

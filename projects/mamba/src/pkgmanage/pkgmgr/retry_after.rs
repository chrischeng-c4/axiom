// retry_after.rs — HTTP `Retry-After` header parser (RFC 9110 §10.2.3).
//
// Servers signal back-off on HTTP 429 (Too Many Requests) and 503 (Service
// Unavailable) by sending a `Retry-After` header. uv honors this when
// hitting PyPI's rate limiter; mamba's HTTP layer wants the same surface.
//
// Per RFC 9110:
//
//     Retry-After = HTTP-date / delta-seconds
//     delta-seconds = 1*DIGIT
//
// HTTP-date has three accepted forms (§5.6.7):
//   * IMF-fixdate    (MUST be the only form senders generate):
//                    `Sun, 06 Nov 1994 08:49:37 GMT`
//   * obs-RFC850     `Sunday, 06-Nov-94 08:49:37 GMT`
//   * obs-asctime    `Sun Nov  6 08:49:37 1994`
//
// This module implements IMF-fixdate (the SHOULD form) plus delta-seconds.
// Legacy date forms are recognised only enough to reject them as
// `ParseError`s — callers can either retry without back-off or fall back
// to a default delay. In practice, every real PyPI / index server emits
// either an integer or IMF-fixdate.

use crate::pkgmanage::pkgmgr::types::IndexError;

/// Parsed Retry-After value.
///
/// `Seconds` is a relative delay; `Date` is an absolute UNIX-epoch
/// timestamp the caller compares to "now".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetryAfter {
    Seconds(u64),
    Date(u64),
}

/// Parse the value of a `Retry-After` header.
pub fn parse_retry_after(header: &str) -> Result<RetryAfter, IndexError> {
    let s = header.trim();
    if s.is_empty() {
        return Err(IndexError::ParseError {
            url: "Retry-After".into(),
            detail: "empty Retry-After header".into(),
        });
    }
    if s.chars().all(|c| c.is_ascii_digit()) {
        let n: u64 = s
            .parse()
            .map_err(|e: std::num::ParseIntError| IndexError::ParseError {
                url: "Retry-After".into(),
                detail: format!("Retry-After delta-seconds out of range: {e}"),
            })?;
        return Ok(RetryAfter::Seconds(n));
    }
    // Fall through to HTTP-date.
    parse_imf_fixdate(s).map(RetryAfter::Date)
}

/// Compute the back-off duration in seconds against a caller-supplied
/// `now_unix` timestamp. `RetryAfter::Date` values in the past saturate
/// to 0 (retry immediately).
pub fn compute_delay_secs(retry: RetryAfter, now_unix: u64) -> u64 {
    match retry {
        RetryAfter::Seconds(n) => n,
        RetryAfter::Date(when) => when.saturating_sub(now_unix),
    }
}

// ---- IMF-fixdate parser -------------------------------------------------
//
// Format: `Sun, 06 Nov 1994 08:49:37 GMT`
//
//   * Weekday + comma + space: discarded — we don't validate the name
//     against the date; servers are not consistent about this.
//   * Day of month: 2-digit, zero-padded.
//   * Month abbreviation: 3 chars, case-sensitive per the spec but we
//     accept any case.
//   * Year: 4-digit.
//   * Time: HH:MM:SS, all zero-padded.
//   * Trailing literal `GMT`.

fn parse_imf_fixdate(s: &str) -> Result<u64, IndexError> {
    // Skip weekday prefix up to and including the first comma.
    let rest = match s.split_once(',') {
        Some((_wkd, r)) => r.trim_start(),
        None => return err("missing weekday-comma prefix"),
    };
    let parts: Vec<&str> = rest.split_whitespace().collect();
    if parts.len() != 5 {
        return err(&format!(
            "expected 'DD Mon YYYY HH:MM:SS GMT', got {} tokens",
            parts.len()
        ));
    }
    let day: u32 = parts[0].parse().map_err(|_| pe("invalid day"))?;
    let month = parse_month(parts[1])?;
    let year: i32 = parts[2].parse().map_err(|_| pe("invalid year"))?;
    let (hh, mm, ss) = parse_hms(parts[3])?;
    if !parts[4].eq_ignore_ascii_case("GMT") {
        return err(&format!("expected 'GMT' suffix, got {:?}", parts[4]));
    }
    if !(1..=31).contains(&day) {
        return err("day out of range");
    }
    if hh >= 24 || mm >= 60 || ss > 60 {
        return err("time component out of range");
    }
    let days = days_from_civil(year, month, day);
    let epoch_seconds = (days as i64) * 86_400 + (hh as i64) * 3600 + (mm as i64) * 60 + ss as i64;
    if epoch_seconds < 0 {
        return err("pre-epoch HTTP-date not supported");
    }
    Ok(epoch_seconds as u64)
}

fn parse_month(s: &str) -> Result<u32, IndexError> {
    match s.to_ascii_lowercase().as_str() {
        "jan" => Ok(1),
        "feb" => Ok(2),
        "mar" => Ok(3),
        "apr" => Ok(4),
        "may" => Ok(5),
        "jun" => Ok(6),
        "jul" => Ok(7),
        "aug" => Ok(8),
        "sep" => Ok(9),
        "oct" => Ok(10),
        "nov" => Ok(11),
        "dec" => Ok(12),
        other => Err(pe(&format!("unknown month {other:?}"))),
    }
}

fn parse_hms(s: &str) -> Result<(u32, u32, u32), IndexError> {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() != 3 {
        return Err(pe("time must be HH:MM:SS"));
    }
    let h: u32 = parts[0].parse().map_err(|_| pe("invalid HH"))?;
    let m: u32 = parts[1].parse().map_err(|_| pe("invalid MM"))?;
    let s: u32 = parts[2].parse().map_err(|_| pe("invalid SS"))?;
    Ok((h, m, s))
}

// Days from the civil 1970-01-01 epoch, using Howard Hinnant's algorithm.
// Result is negative for pre-1970 dates.
fn days_from_civil(y: i32, m: u32, d: u32) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = (y - era * 400) as i64;
    let m = m as i64;
    let d = d as i64;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    (era as i64) * 146_097 + doe - 719_468
}

fn err(msg: &str) -> Result<u64, IndexError> {
    Err(pe(msg))
}

fn pe(msg: &str) -> IndexError {
    IndexError::ParseError {
        url: "Retry-After".into(),
        detail: msg.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delta_seconds_zero() {
        assert_eq!(parse_retry_after("0").unwrap(), RetryAfter::Seconds(0));
    }

    #[test]
    fn delta_seconds_typical() {
        assert_eq!(parse_retry_after("120").unwrap(), RetryAfter::Seconds(120));
    }

    #[test]
    fn delta_seconds_trims_whitespace() {
        assert_eq!(
            parse_retry_after("   30   ").unwrap(),
            RetryAfter::Seconds(30)
        );
    }

    #[test]
    fn delta_seconds_overflow_rejected() {
        // u64::MAX + 1
        let err = parse_retry_after("18446744073709551616").unwrap_err();
        assert!(err.to_string().contains("delta-seconds"));
    }

    #[test]
    fn empty_rejected() {
        let err = parse_retry_after("   ").unwrap_err();
        assert!(err.to_string().contains("empty"));
    }

    #[test]
    fn negative_seconds_rejected() {
        // `-` is not an ASCII digit, so this falls through to date parser.
        let err = parse_retry_after("-5").unwrap_err();
        // Date parser bails on missing weekday comma.
        let s = err.to_string();
        assert!(s.contains("comma") || s.contains("expected"), "got {s}");
    }

    #[test]
    fn imf_fixdate_basic() {
        // 1994-11-06 08:49:37 GMT — RFC 9110's own example.
        // Expected Unix epoch: 784111777.
        let v = parse_retry_after("Sun, 06 Nov 1994 08:49:37 GMT").unwrap();
        assert_eq!(v, RetryAfter::Date(784_111_777));
    }

    #[test]
    fn imf_fixdate_unix_epoch() {
        // 1970-01-01T00:00:00Z.
        let v = parse_retry_after("Thu, 01 Jan 1970 00:00:00 GMT").unwrap();
        assert_eq!(v, RetryAfter::Date(0));
    }

    #[test]
    fn imf_fixdate_known_date_2024() {
        // 2024-01-01T00:00:00Z = 1704067200.
        let v = parse_retry_after("Mon, 01 Jan 2024 00:00:00 GMT").unwrap();
        assert_eq!(v, RetryAfter::Date(1_704_067_200));
    }

    #[test]
    fn imf_fixdate_weekday_not_validated() {
        // A clearly wrong weekday is tolerated — we don't enforce it.
        let v = parse_retry_after("Foo, 01 Jan 2024 00:00:00 GMT").unwrap();
        assert_eq!(v, RetryAfter::Date(1_704_067_200));
    }

    #[test]
    fn imf_fixdate_month_case_insensitive() {
        let v = parse_retry_after("Mon, 01 jAn 2024 00:00:00 GMT").unwrap();
        assert_eq!(v, RetryAfter::Date(1_704_067_200));
    }

    #[test]
    fn imf_fixdate_gmt_case_insensitive() {
        let v = parse_retry_after("Mon, 01 Jan 2024 00:00:00 gmt").unwrap();
        assert_eq!(v, RetryAfter::Date(1_704_067_200));
    }

    #[test]
    fn imf_fixdate_bad_month_rejected() {
        let err = parse_retry_after("Sun, 06 Xxx 1994 08:49:37 GMT").unwrap_err();
        assert!(err.to_string().contains("unknown month"));
    }

    #[test]
    fn imf_fixdate_bad_time_format_rejected() {
        let err = parse_retry_after("Sun, 06 Nov 1994 08-49-37 GMT").unwrap_err();
        assert!(err.to_string().contains("HH:MM:SS"));
    }

    #[test]
    fn imf_fixdate_missing_gmt_rejected() {
        let err = parse_retry_after("Sun, 06 Nov 1994 08:49:37 UTC").unwrap_err();
        assert!(err.to_string().contains("GMT"));
    }

    #[test]
    fn imf_fixdate_day_out_of_range_rejected() {
        let err = parse_retry_after("Sun, 32 Nov 1994 00:00:00 GMT").unwrap_err();
        assert!(err.to_string().contains("day"));
    }

    #[test]
    fn imf_fixdate_hour_out_of_range_rejected() {
        let err = parse_retry_after("Sun, 06 Nov 1994 25:00:00 GMT").unwrap_err();
        assert!(err.to_string().contains("time component"));
    }

    #[test]
    fn compute_delay_seconds_passthrough() {
        assert_eq!(compute_delay_secs(RetryAfter::Seconds(42), 100), 42);
    }

    #[test]
    fn compute_delay_date_future() {
        // when = 100, now = 60 → delay = 40
        assert_eq!(compute_delay_secs(RetryAfter::Date(100), 60), 40);
    }

    #[test]
    fn compute_delay_date_past_clamps_to_zero() {
        assert_eq!(compute_delay_secs(RetryAfter::Date(50), 100), 0);
    }

    #[test]
    fn compute_delay_date_exactly_now_is_zero() {
        assert_eq!(compute_delay_secs(RetryAfter::Date(100), 100), 0);
    }
}

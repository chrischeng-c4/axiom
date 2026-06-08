//! Time and size humanization utilities.
//!
//! Provides functions to convert durations and byte sizes into human-readable strings.
//!
//! # Functions
//!
//! | Function | Example |
//! |----------|---------|
//! | `naturaltime` | `3600` -> `"an hour ago"` |
//! | `naturaldelta` | `3600` -> `"an hour"` |
//! | `naturalsize` | `1_048_576` -> `"1.0 MB"` |

/// Convert a number of seconds into a human-readable relative time string.
///
/// Positive values represent past time ("ago"), negative values represent
/// future time ("from now").
///
/// # Examples
///
/// ```
/// use cclab_util::humanize::time_size::naturaltime;
///
/// assert_eq!(naturaltime(0.0), "now");
/// assert_eq!(naturaltime(1.0), "a second ago");
/// assert_eq!(naturaltime(30.0), "30 seconds ago");
/// assert_eq!(naturaltime(90.0), "a minute ago");
/// assert_eq!(naturaltime(3600.0), "an hour ago");
/// assert_eq!(naturaltime(-60.0), "a minute from now");
/// ```
pub fn naturaltime(seconds: f64) -> String {
    if seconds.abs() < 0.5 {
        return "now".to_string();
    }

    let delta = naturaldelta(seconds.abs());
    if seconds > 0.0 {
        format!("{} ago", delta)
    } else {
        format!("{} from now", delta)
    }
}

/// Convert a number of seconds into a human-readable duration string.
///
/// Unlike `naturaltime`, this does not add "ago" or "from now" — it just
/// describes the duration.
///
/// # Examples
///
/// ```
/// use cclab_util::humanize::time_size::naturaldelta;
///
/// assert_eq!(naturaldelta(0.0), "a moment");
/// assert_eq!(naturaldelta(1.0), "a second");
/// assert_eq!(naturaldelta(30.0), "30 seconds");
/// assert_eq!(naturaldelta(60.0), "a minute");
/// assert_eq!(naturaldelta(90.0), "a minute");
/// assert_eq!(naturaldelta(120.0), "2 minutes");
/// assert_eq!(naturaldelta(3600.0), "an hour");
/// assert_eq!(naturaldelta(7200.0), "2 hours");
/// assert_eq!(naturaldelta(86400.0), "a day");
/// assert_eq!(naturaldelta(86400.0 * 365.25), "a year");
/// ```
pub fn naturaldelta(seconds: f64) -> String {
    let abs_seconds = seconds.abs();

    if abs_seconds < 0.5 {
        return "a moment".to_string();
    }

    let secs = abs_seconds as u64;

    if secs < 2 {
        return "a second".to_string();
    }

    if secs < 60 {
        return format!("{} seconds", secs);
    }

    let minutes = secs / 60;
    if minutes < 2 {
        return "a minute".to_string();
    }
    if minutes < 60 {
        return format!("{} minutes", minutes);
    }

    let hours = minutes / 60;
    if hours < 2 {
        return "an hour".to_string();
    }
    if hours < 24 {
        return format!("{} hours", hours);
    }

    let days = hours / 24;
    if days < 2 {
        return "a day".to_string();
    }
    if days < 30 {
        return format!("{} days", days);
    }

    let months = days / 30;
    if months < 2 {
        return "a month".to_string();
    }
    if months < 12 {
        return format!("{} months", months);
    }

    let years = days as f64 / 365.25;
    if years < 2.0 {
        return "a year".to_string();
    }

    format!("{} years", years as u64)
}

/// Binary size units (base 1024).
const BINARY_UNITS: &[&str] = &["Bytes", "KB", "MB", "GB", "TB", "PB", "EB"];

/// Decimal size units (base 1000).
const DECIMAL_UNITS: &[&str] = &["Bytes", "kB", "MB", "GB", "TB", "PB", "EB"];

/// Format for size display.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SizeFormat {
    /// Binary format (base 1024): KB, MB, GB, etc.
    Binary,
    /// Decimal format (base 1000): kB, MB, GB, etc.
    Decimal,
    /// GNU format: K, M, G, etc.
    Gnu,
}

/// Convert a byte count into a human-readable file size string.
///
/// # Arguments
///
/// * `bytes` - Number of bytes
/// * `binary` - If true, use base-1024 (KB); otherwise base-1000 (kB)
///
/// # Examples
///
/// ```
/// use cclab_util::humanize::time_size::naturalsize;
///
/// assert_eq!(naturalsize(0, false), "0 Bytes");
/// assert_eq!(naturalsize(1, false), "1 Byte");
/// assert_eq!(naturalsize(1000, false), "1.0 kB");
/// assert_eq!(naturalsize(1024, true), "1.0 KB");
/// assert_eq!(naturalsize(1_048_576, true), "1.0 MB");
/// assert_eq!(naturalsize(1_000_000, false), "1.0 MB");
/// assert_eq!(naturalsize(1_500_000_000, false), "1.5 GB");
/// ```
pub fn naturalsize(bytes: u64, binary: bool) -> String {
    if bytes == 0 {
        return "0 Bytes".to_string();
    }
    if bytes == 1 {
        return "1 Byte".to_string();
    }

    let (base, units): (f64, &[&str]) = if binary {
        (1024.0, BINARY_UNITS)
    } else {
        (1000.0, DECIMAL_UNITS)
    };

    let mut value = bytes as f64;
    for (i, unit) in units.iter().enumerate() {
        if value < base || i == units.len() - 1 {
            if i == 0 {
                return format!("{} {}", bytes, unit);
            }
            return format!("{:.1} {}", value, unit);
        }
        value /= base;
    }

    // Fallback (should not reach here)
    format!("{} Bytes", bytes)
}

/// Convert a byte count with custom format.
///
/// # Examples
///
/// ```
/// use cclab_util::humanize::time_size::{naturalsize_fmt, SizeFormat};
///
/// assert_eq!(naturalsize_fmt(1024, SizeFormat::Binary), "1.0 KB");
/// assert_eq!(naturalsize_fmt(1000, SizeFormat::Decimal), "1.0 kB");
/// assert_eq!(naturalsize_fmt(1024, SizeFormat::Gnu), "1.0K");
/// ```
pub fn naturalsize_fmt(bytes: u64, format: SizeFormat) -> String {
    match format {
        SizeFormat::Binary => naturalsize(bytes, true),
        SizeFormat::Decimal => naturalsize(bytes, false),
        SizeFormat::Gnu => naturalsize_gnu(bytes),
    }
}

fn naturalsize_gnu(bytes: u64) -> String {
    const GNU_UNITS: &[&str] = &["", "K", "M", "G", "T", "P", "E"];

    if bytes == 0 {
        return "0".to_string();
    }

    let mut value = bytes as f64;
    for (i, unit) in GNU_UNITS.iter().enumerate() {
        if value < 1024.0 || i == GNU_UNITS.len() - 1 {
            if i == 0 {
                return format!("{}", bytes);
            }
            return format!("{:.1}{}", value, unit);
        }
        value /= 1024.0;
    }

    format!("{}", bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_naturaltime_past() {
        assert_eq!(naturaltime(0.0), "now");
        assert_eq!(naturaltime(0.3), "now");
        assert_eq!(naturaltime(1.0), "a second ago");
        assert_eq!(naturaltime(30.0), "30 seconds ago");
        assert_eq!(naturaltime(60.0), "a minute ago");
        assert_eq!(naturaltime(120.0), "2 minutes ago");
        assert_eq!(naturaltime(3600.0), "an hour ago");
        assert_eq!(naturaltime(7200.0), "2 hours ago");
        assert_eq!(naturaltime(86400.0), "a day ago");
        assert_eq!(naturaltime(86400.0 * 30.0), "a month ago");
        assert_eq!(naturaltime(86400.0 * 365.25), "a year ago");
        assert_eq!(naturaltime(86400.0 * 731.0), "2 years ago");
    }

    #[test]
    fn test_naturaltime_future() {
        assert_eq!(naturaltime(-1.0), "a second from now");
        assert_eq!(naturaltime(-60.0), "a minute from now");
        assert_eq!(naturaltime(-3600.0), "an hour from now");
    }

    #[test]
    fn test_naturaldelta() {
        assert_eq!(naturaldelta(0.0), "a moment");
        assert_eq!(naturaldelta(1.0), "a second");
        assert_eq!(naturaldelta(45.0), "45 seconds");
        assert_eq!(naturaldelta(60.0), "a minute");
        assert_eq!(naturaldelta(90.0), "a minute");
        assert_eq!(naturaldelta(120.0), "2 minutes");
        assert_eq!(naturaldelta(3600.0), "an hour");
        assert_eq!(naturaldelta(7200.0), "2 hours");
        assert_eq!(naturaldelta(86400.0), "a day");
        assert_eq!(naturaldelta(86400.0 * 2.0), "2 days");
        assert_eq!(naturaldelta(86400.0 * 15.0), "15 days");
        assert_eq!(naturaldelta(86400.0 * 30.0), "a month");
        assert_eq!(naturaldelta(86400.0 * 60.0), "2 months");
        assert_eq!(naturaldelta(86400.0 * 365.25), "a year");
        assert_eq!(naturaldelta(86400.0 * 731.0), "2 years");
    }

    #[test]
    fn test_naturalsize_decimal() {
        assert_eq!(naturalsize(0, false), "0 Bytes");
        assert_eq!(naturalsize(1, false), "1 Byte");
        assert_eq!(naturalsize(999, false), "999 Bytes");
        assert_eq!(naturalsize(1000, false), "1.0 kB");
        assert_eq!(naturalsize(1500, false), "1.5 kB");
        assert_eq!(naturalsize(1_000_000, false), "1.0 MB");
        assert_eq!(naturalsize(1_500_000, false), "1.5 MB");
        assert_eq!(naturalsize(1_000_000_000, false), "1.0 GB");
        assert_eq!(naturalsize(1_500_000_000, false), "1.5 GB");
    }

    #[test]
    fn test_naturalsize_binary() {
        assert_eq!(naturalsize(0, true), "0 Bytes");
        assert_eq!(naturalsize(1, true), "1 Byte");
        assert_eq!(naturalsize(1023, true), "1023 Bytes");
        assert_eq!(naturalsize(1024, true), "1.0 KB");
        assert_eq!(naturalsize(1536, true), "1.5 KB");
        assert_eq!(naturalsize(1_048_576, true), "1.0 MB");
        assert_eq!(naturalsize(1_073_741_824, true), "1.0 GB");
    }

    #[test]
    fn test_naturalsize_gnu() {
        assert_eq!(naturalsize_fmt(0, SizeFormat::Gnu), "0");
        assert_eq!(naturalsize_fmt(1024, SizeFormat::Gnu), "1.0K");
        assert_eq!(naturalsize_fmt(1_048_576, SizeFormat::Gnu), "1.0M");
    }
}

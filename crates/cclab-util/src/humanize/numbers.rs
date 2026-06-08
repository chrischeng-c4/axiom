//! Number humanization utilities.
//!
//! Provides functions to convert numbers into human-readable strings.
//!
//! # Functions
//!
//! | Function | Example |
//! |----------|---------|
//! | `intcomma` | `1000` -> `"1,000"` |
//! | `intword` | `1_000_000` -> `"1.0 million"` |
//! | `ordinal` | `1` -> `"1st"` |
//! | `apnumber` | `1` -> `"one"` |

/// Format an integer with commas as thousands separators.
///
/// # Examples
///
/// ```
/// use cclab_util::humanize::numbers::intcomma;
///
/// assert_eq!(intcomma(1000), "1,000");
/// assert_eq!(intcomma(1_000_000), "1,000,000");
/// assert_eq!(intcomma(100), "100");
/// assert_eq!(intcomma(-1234567), "-1,234,567");
/// ```
pub fn intcomma(value: i64) -> String {
    if value == 0 {
        return "0".to_string();
    }

    let negative = value < 0;
    let n = if negative {
        // Handle i64::MIN carefully
        if value == i64::MIN {
            let s = intcomma_unsigned(i64::MAX as u64 + 1);
            return format!("-{}", s);
        }
        (-value) as u64
    } else {
        value as u64
    };

    let result = intcomma_unsigned(n);
    if negative {
        format!("-{}", result)
    } else {
        result
    }
}

fn intcomma_unsigned(n: u64) -> String {
    let s = n.to_string();
    let bytes = s.as_bytes();
    let len = bytes.len();

    if len <= 3 {
        return s;
    }

    let mut result = String::with_capacity(len + (len - 1) / 3);
    // Number of digits in the first (leftmost) group: 1, 2, or 3
    let first_group = match len % 3 {
        0 => 3,
        r => r,
    };

    for (i, &b) in bytes.iter().enumerate() {
        if i > 0 && i >= first_group && (i - first_group) % 3 == 0 {
            result.push(',');
        }
        result.push(b as char);
    }

    result
}

/// Format an integer with commas, supporting float input.
///
/// # Examples
///
/// ```
/// use cclab_util::humanize::numbers::intcomma_f64;
///
/// assert_eq!(intcomma_f64(1000.5), "1,000.5");
/// assert_eq!(intcomma_f64(-1234.99), "-1,234.99");
/// ```
pub fn intcomma_f64(value: f64) -> String {
    let s = format!("{}", value);

    if let Some(dot_pos) = s.find('.') {
        let int_str = &s[..dot_pos];
        let decimal_part = &s[dot_pos..]; // includes the '.'

        // Parse int part, handling the sign
        let int_val: i64 = int_str.parse().unwrap_or(0);
        format!("{}{}", intcomma(int_val), decimal_part)
    } else {
        let int_val: i64 = s.parse().unwrap_or(0);
        intcomma(int_val)
    }
}

/// Convert a large integer to a friendly text representation.
///
/// Powers of 10 are mapped to word suffixes:
/// - 10^6: million
/// - 10^9: billion
/// - 10^12: trillion
/// - 10^15: quadrillion
/// - 10^18: quintillion
///
/// Values below 1 million are returned with `intcomma`.
///
/// # Examples
///
/// ```
/// use cclab_util::humanize::numbers::intword;
///
/// assert_eq!(intword(100), "100");
/// assert_eq!(intword(1_000_000), "1.0 million");
/// assert_eq!(intword(1_200_000), "1.2 million");
/// assert_eq!(intword(1_000_000_000), "1.0 billion");
/// assert_eq!(intword(1_500_000_000_000), "1.5 trillion");
/// ```
pub fn intword(value: i64) -> String {
    let abs_value = value.unsigned_abs();

    if abs_value < 1_000_000 {
        return intcomma(value);
    }

    let negative = value < 0;
    let powers: &[(u64, &str)] = &[
        (1_000_000_000_000_000_000, "quintillion"),
        (1_000_000_000_000_000, "quadrillion"),
        (1_000_000_000_000, "trillion"),
        (1_000_000_000, "billion"),
        (1_000_000, "million"),
    ];

    for &(divisor, word) in powers {
        if abs_value >= divisor {
            let float_val = abs_value as f64 / divisor as f64;
            let formatted = format!("{:.1} {}", float_val, word);
            return if negative {
                format!("-{}", formatted)
            } else {
                formatted
            };
        }
    }

    intcomma(value)
}

/// Convert an integer to its ordinal string representation.
///
/// # Examples
///
/// ```
/// use cclab_util::humanize::numbers::ordinal;
///
/// assert_eq!(ordinal(1), "1st");
/// assert_eq!(ordinal(2), "2nd");
/// assert_eq!(ordinal(3), "3rd");
/// assert_eq!(ordinal(4), "4th");
/// assert_eq!(ordinal(11), "11th");
/// assert_eq!(ordinal(12), "12th");
/// assert_eq!(ordinal(13), "13th");
/// assert_eq!(ordinal(21), "21st");
/// assert_eq!(ordinal(22), "22nd");
/// assert_eq!(ordinal(23), "23rd");
/// assert_eq!(ordinal(111), "111th");
/// ```
pub fn ordinal(value: i64) -> String {
    let abs_val = value.unsigned_abs();
    let suffix = match (abs_val % 10, abs_val % 100) {
        (1, 11) => "th",
        (2, 12) => "th",
        (3, 13) => "th",
        (1, _) => "st",
        (2, _) => "nd",
        (3, _) => "rd",
        _ => "th",
    };
    format!("{}{}", value, suffix)
}

/// For small numbers (1-9), return the English word. Otherwise return digits.
///
/// Per Associated Press style, numbers 1 through 9 should be spelled out.
///
/// # Examples
///
/// ```
/// use cclab_util::humanize::numbers::apnumber;
///
/// assert_eq!(apnumber(1), "one");
/// assert_eq!(apnumber(2), "two");
/// assert_eq!(apnumber(9), "nine");
/// assert_eq!(apnumber(10), "10");
/// assert_eq!(apnumber(0), "0");
/// assert_eq!(apnumber(-1), "-1");
/// ```
pub fn apnumber(value: i64) -> String {
    const WORDS: [&str; 9] = [
        "one", "two", "three", "four", "five", "six", "seven", "eight", "nine",
    ];

    if (1..=9).contains(&value) {
        WORDS[(value - 1) as usize].to_string()
    } else {
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intcomma() {
        assert_eq!(intcomma(0), "0");
        assert_eq!(intcomma(100), "100");
        assert_eq!(intcomma(1000), "1,000");
        assert_eq!(intcomma(10000), "10,000");
        assert_eq!(intcomma(100000), "100,000");
        assert_eq!(intcomma(1_000_000), "1,000,000");
        assert_eq!(intcomma(1_234_567_890), "1,234,567,890");
        assert_eq!(intcomma(-1000), "-1,000");
        assert_eq!(intcomma(-1_234_567), "-1,234,567");
    }

    #[test]
    fn test_intcomma_f64() {
        assert_eq!(intcomma_f64(1000.0), "1,000");
        assert_eq!(intcomma_f64(1000.5), "1,000.5");
        assert_eq!(intcomma_f64(-1234.99), "-1,234.99");
    }

    #[test]
    fn test_intword() {
        assert_eq!(intword(100), "100");
        assert_eq!(intword(999_999), "999,999");
        assert_eq!(intword(1_000_000), "1.0 million");
        assert_eq!(intword(1_200_000), "1.2 million");
        assert_eq!(intword(1_000_000_000), "1.0 billion");
        assert_eq!(intword(1_500_000_000_000), "1.5 trillion");
        assert_eq!(intword(-2_000_000), "-2.0 million");
    }

    #[test]
    fn test_ordinal() {
        assert_eq!(ordinal(1), "1st");
        assert_eq!(ordinal(2), "2nd");
        assert_eq!(ordinal(3), "3rd");
        assert_eq!(ordinal(4), "4th");
        assert_eq!(ordinal(10), "10th");
        assert_eq!(ordinal(11), "11th");
        assert_eq!(ordinal(12), "12th");
        assert_eq!(ordinal(13), "13th");
        assert_eq!(ordinal(21), "21st");
        assert_eq!(ordinal(22), "22nd");
        assert_eq!(ordinal(23), "23rd");
        assert_eq!(ordinal(100), "100th");
        assert_eq!(ordinal(101), "101st");
        assert_eq!(ordinal(111), "111th");
        assert_eq!(ordinal(112), "112th");
        assert_eq!(ordinal(113), "113th");
    }

    #[test]
    fn test_apnumber() {
        assert_eq!(apnumber(0), "0");
        assert_eq!(apnumber(1), "one");
        assert_eq!(apnumber(5), "five");
        assert_eq!(apnumber(9), "nine");
        assert_eq!(apnumber(10), "10");
        assert_eq!(apnumber(100), "100");
        assert_eq!(apnumber(-1), "-1");
    }
}

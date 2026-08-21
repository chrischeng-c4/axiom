//! TOTP/HOTP one-time password generation and verification.
//!
//! Compatible with Google Authenticator, Authy, etc.

use crate::hash::hmac_sha1;
use std::time::{SystemTime, UNIX_EPOCH};

const DEFAULT_DIGITS: u32 = 6;
const DEFAULT_PERIOD: u64 = 30;

/// Dynamic truncation per RFC 4226.
fn dynamic_truncation(hmac_result: &[u8]) -> u32 {
    let offset = (hmac_result[hmac_result.len() - 1] & 0x0f) as usize;
    ((hmac_result[offset] as u32 & 0x7f) << 24)
        | ((hmac_result[offset + 1] as u32) << 16)
        | ((hmac_result[offset + 2] as u32) << 8)
        | (hmac_result[offset + 3] as u32)
}

/// Generate an HOTP code (RFC 4226).
pub fn hotp_generate(secret: &[u8], counter: u64, digits: Option<u32>) -> String {
    let digits = digits.unwrap_or(DEFAULT_DIGITS);
    let hmac = hmac_sha1(secret, &counter.to_be_bytes());
    let code = dynamic_truncation(&hmac) % 10u32.pow(digits);
    format!("{:0>width$}", code, width = digits as usize)
}

/// Verify an HOTP code.
pub fn hotp_verify(secret: &[u8], counter: u64, code: &str, digits: Option<u32>) -> bool {
    let expected = hotp_generate(secret, counter, digits);
    constant_time_eq(expected.as_bytes(), code.as_bytes())
}

/// Generate a TOTP code (RFC 6238).
pub fn totp_generate(secret: &[u8], period: Option<u64>, digits: Option<u32>) -> String {
    let period = period.unwrap_or(DEFAULT_PERIOD);
    let counter = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        / period;
    hotp_generate(secret, counter, digits)
}

/// Verify a TOTP code with optional time window tolerance.
///
/// `skew` controls how many periods before/after current time to accept (default 1).
pub fn totp_verify(
    secret: &[u8],
    code: &str,
    period: Option<u64>,
    digits: Option<u32>,
    skew: Option<u32>,
) -> bool {
    let period = period.unwrap_or(DEFAULT_PERIOD);
    let skew = skew.unwrap_or(1) as u64;
    let counter = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        / period;
    for offset in 0..=skew {
        if hotp_verify(secret, counter + offset, code, digits) {
            return true;
        }
        if offset > 0 && counter >= offset && hotp_verify(secret, counter - offset, code, digits) {
            return true;
        }
    }
    false
}

/// Constant-time comparison to prevent timing attacks.
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hotp_rfc4226() {
        // RFC 4226 test vectors (secret = "12345678901234567890")
        let secret = b"12345678901234567890";
        let expected = [
            "755224", "287082", "359152", "969429", "338314", "254676", "287922", "162583",
            "399871", "520489",
        ];
        for (counter, exp) in expected.iter().enumerate() {
            assert_eq!(hotp_generate(secret, counter as u64, Some(6)), *exp);
        }
    }

    #[test]
    fn test_hotp_verify() {
        let secret = b"12345678901234567890";
        assert!(hotp_verify(secret, 0, "755224", Some(6)));
        assert!(!hotp_verify(secret, 0, "000000", Some(6)));
    }

    #[test]
    fn test_totp_generate() {
        let secret = b"testsecret";
        let code = totp_generate(secret, Some(30), Some(6));
        assert_eq!(code.len(), 6);
        assert!(code.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn test_totp_verify_current() {
        let secret = b"testsecret";
        let code = totp_generate(secret, Some(30), Some(6));
        assert!(totp_verify(secret, &code, Some(30), Some(6), Some(0)));
    }
}

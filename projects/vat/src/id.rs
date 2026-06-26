// SPEC-MANAGED: projects/vat/tech-design/semantic/source/projects-vat-src-id-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! Vat identifiers.
//!
//! An id is short, lowercase, and greppable: `vat-` + a base36 stamp derived
//! from the wall clock and pid. Collisions are astronomically unlikely for a
//! local, single-user tool; if two vats ever land on the same id, [`store`]
//! refuses to clobber an existing directory.
//!
//! [`store`]: crate::store

use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

/// Generate a fresh vat id, e.g. `vat-7f3k1q9`.
/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-id-rs.md#source
pub fn fresh() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    // Mix in the pid so two vats created within the same nanosecond tick
    // (e.g. a fork burst) still diverge.
    let mixed = nanos ^ ((process::id() as u128) << 80);
    format!("vat-{}", base36(mixed as u64 & 0xff_ffff_ffff))
}

/// Lowercase base36 of a u64 (no leading-zero padding; ids are opaque).
fn base36(mut n: u64) -> String {
    const ALPHABET: &[u8; 36] = b"0123456789abcdefghijklmnopqrstuvwxyz";
    if n == 0 {
        return "0".to_string();
    }
    let mut buf = Vec::new();
    while n > 0 {
        buf.push(ALPHABET[(n % 36) as usize]);
        n /= 36;
    }
    buf.reverse();
    String::from_utf8(buf).expect("base36 alphabet is ascii")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fresh_ids_have_prefix_and_differ() {
        let a = fresh();
        assert!(a.starts_with("vat-"), "got {a}");
        // The clock advances between calls, so ids differ in practice.
        let b = fresh();
        assert_ne!(a, b);
    }

    #[test]
    fn base36_is_stable() {
        assert_eq!(base36(0), "0");
        assert_eq!(base36(35), "z");
        assert_eq!(base36(36), "10");
    }
}
// CODEGEN-END

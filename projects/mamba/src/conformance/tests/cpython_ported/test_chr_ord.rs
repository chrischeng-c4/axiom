//! Py3.12 conformance tests for `chr` and `ord` (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_builtin.py — chr/ord
//! sections):
//!   ASCII roundtrip in both directions and arithmetic on the
//!   codepoint produced by `ord`.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_chr_produces_expected_ascii_characters() {
    let out = jit_capture(
        r#"print(chr(65))
print(chr(97))
print(chr(48))
"#,
    );
    assert_output(&out, "A\na\n0\n");
}

#[test]
fn test_ord_returns_codepoint_of_single_char() {
    let out = jit_capture(
        r#"print(ord("A"))
print(ord("a"))
print(ord("0"))
"#,
    );
    assert_output(&out, "65\n97\n48\n");
}

#[test]
fn test_chr_ord_arithmetic_roundtrip() {
    let out = jit_capture(
        r#"print(chr(ord("A") + 1))
print(chr(ord("a") + 2))
print(ord(chr(100)))
"#,
    );
    assert_output(&out, "B\nc\n100\n");
}

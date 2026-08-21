//! Py3.12 conformance tests for string case and strip methods
//! (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_unicode.py — case and
//! strip sections):
//!   `upper`/`lower`/`swapcase`, default whitespace `strip`/`lstrip`/
//!   `rstrip`, and `strip` with a custom character set.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_str_upper_lower_swapcase() {
    let out = jit_capture(
        r#"print("Hello".upper())
print("Hello".lower())
print("Hello".swapcase())
"#,
    );
    assert_output(&out, "HELLO\nhello\nhELLO\n");
}

#[test]
fn test_str_strip_default_whitespace() {
    let out = jit_capture(
        r#"print(repr("  hi  ".strip()))
print(repr("  hi  ".lstrip()))
print(repr("  hi  ".rstrip()))
"#,
    );
    assert_output(&out, "'hi'\n'hi  '\n'  hi'\n");
}

#[test]
fn test_str_strip_with_custom_chars() {
    let out = jit_capture(
        r#"print("xxhixx".strip("x"))
print("---note---".strip("-"))
print("abcXYZcba".strip("abc"))
"#,
    );
    assert_output(&out, "hi\nnote\nXYZ\n");
}

//! Py3.12 conformance tests for string predicate methods (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_unicode.py — `is*`
//! predicate sections):
//!   `isalpha`, `isdigit`, `isalnum`, `isspace`, `islower`, `isupper`,
//!   and the `title()` transform.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_str_isalpha_isdigit_isalnum() {
    let out = jit_capture(
        r#"print("abc".isalpha())
print("HELLO".isalpha())
print("123".isdigit())
print("a1".isalnum())
print("a 1".isalnum())
"#,
    );
    assert_output(&out, "True\nTrue\nTrue\nTrue\nFalse\n");
}

#[test]
fn test_str_isspace() {
    let out = jit_capture(
        r#"print("  ".isspace())
print(" \t\n".isspace())
print(" a ".isspace())
"#,
    );
    assert_output(&out, "True\nTrue\nFalse\n");
}

#[test]
fn test_str_islower_isupper() {
    let out = jit_capture(
        r#"print("hello".islower())
print("ABC".islower())
print("ABC".isupper())
print("Abc".isupper())
"#,
    );
    assert_output(&out, "True\nFalse\nTrue\nFalse\n");
}

#[test]
fn test_str_title_transform() {
    let out = jit_capture(
        r#"print("hello world".title())
print("HELLO WORLD".title())
"#,
    );
    assert_output(&out, "Hello World\nHello World\n");
}

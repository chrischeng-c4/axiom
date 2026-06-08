//! Py3.12 conformance tests for `str.capitalize`/`title`/`expandtabs`/
//! `center` and string predicate combinators (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_str.py — case and
//! padding sections):
//!   `capitalize` and `title` casing, `expandtabs` expansion with a
//!   user width, `center` with a non-space fill char, and the
//!   `isdigit`/`isalpha`/`isalnum` predicate trio.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_capitalize_and_title() {
    let out = jit_capture(
        r#"print("hello world".capitalize())
print("hello world".title())
print("PYTHON".capitalize())
"#,
    );
    assert_output(&out, "Hello world\nHello World\nPython\n");
}

#[test]
fn test_expandtabs_and_center_fill() {
    let out = jit_capture(
        r#"print("ab\tcd".expandtabs(4))
print("hello".center(11, "*"))
print("x".center(5, "-"))
"#,
    );
    assert_output(&out, "ab  cd\n***hello***\n--x--\n");
}

#[test]
fn test_predicate_combo() {
    let out = jit_capture(
        r#"print("abc".isalpha())
print("123".isdigit())
print("abc123".isalnum())
print("abc!".isalnum())
"#,
    );
    assert_output(&out, "True\nTrue\nTrue\nFalse\n");
}

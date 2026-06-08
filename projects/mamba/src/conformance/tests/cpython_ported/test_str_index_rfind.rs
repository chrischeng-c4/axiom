//! Py3.12 conformance tests for `str.index` and `str.rfind`
//! (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_unicode.py — search
//! sections):
//!   `index` returns the first occurrence and raises `ValueError` on
//!   miss; `rfind` returns the last occurrence and -1 on miss.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_str_index_returns_first_occurrence() {
    let out = jit_capture(
        r#"print("hello".index("l"))
print("banana".index("a"))
print("hello".index("o"))
"#,
    );
    assert_output(&out, "2\n1\n4\n");
}

#[test]
fn test_str_index_raises_on_miss() {
    let out = jit_capture(
        r#"try:
    print("hello".index("z"))
except ValueError:
    print("not found")
try:
    print("abc".index("d"))
except ValueError:
    print("absent")
"#,
    );
    assert_output(&out, "not found\nabsent\n");
}

#[test]
fn test_str_rfind_last_occurrence_and_miss() {
    let out = jit_capture(
        r#"print("hello".rfind("l"))
print("banana".rfind("a"))
print("hello".rfind("z"))
"#,
    );
    assert_output(&out, "3\n5\n-1\n");
}

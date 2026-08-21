//! Py3.12 conformance tests for string `+`, `*`, `in`, and `len`
//! (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_unicode.py — sequence
//! operator sections):
//!   concatenation, repetition (`s*n` and `n*s`), substring `in`, and
//!   `len` on empty/non-empty strings.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_str_plus_concatenation() {
    let out = jit_capture(
        r#"print("ab" + "cd")
print("foo" + "" + "bar")
print("" + "x")
"#,
    );
    assert_output(&out, "abcd\nfoobar\nx\n");
}

#[test]
fn test_str_repetition_both_orders() {
    let out = jit_capture(
        r#"print("ab" * 3)
print(3 * "x")
print("hi" * 0)
"#,
    );
    assert_output(&out, "ababab\nxxx\n\n");
}

#[test]
fn test_str_in_substring_membership() {
    let out = jit_capture(
        r#"print("l" in "hello")
print("z" in "hello")
print("hel" in "hello")
print("xyz" in "hello")
"#,
    );
    assert_output(&out, "True\nFalse\nTrue\nFalse\n");
}

#[test]
fn test_str_len_on_empty_and_non_empty() {
    let out = jit_capture(
        r#"print(len("hello"))
print(len(""))
print(len("a"))
"#,
    );
    assert_output(&out, "5\n0\n1\n");
}

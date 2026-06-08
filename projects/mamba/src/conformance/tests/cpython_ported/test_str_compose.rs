//! Py3.12 conformance tests for composed string operations (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_str.py — concat,
//! length, repetition, and join sections):
//!   chaining `+`/`len`/`*`/`join` together on identifier-bound strings.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_concat_len_and_repeat_chain() {
    let out = jit_capture(
        r#"a = "hello"
b = "world"
print(a + " " + b)
print(len(a + b))
print((a + b) * 2)
"#,
    );
    assert_output(&out, "hello world\n10\nhelloworldhelloworld\n");
}

#[test]
fn test_join_with_identifier_separator() {
    let out = jit_capture(
        r#"a = "hello"
b = "world"
sep = "-"
print(sep.join([a, b]))
print(("=" * 10))
"#,
    );
    assert_output(&out, "hello-world\n==========\n");
}

#[test]
fn test_repeat_of_concat_and_zero() {
    let out = jit_capture(
        r#"s = "ab"
print(s * 0)
print((s + "c") * 3)
print("x" * 5 + "y")
"#,
    );
    assert_output(&out, "\nabcabcabc\nxxxxxy\n");
}

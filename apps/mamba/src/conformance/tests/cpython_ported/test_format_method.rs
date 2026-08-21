//! Py3.12 conformance tests for `str.format` (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_unicode.py — `format`
//! method sections):
//!   positional/numbered/keyword fields, alignment specifiers
//!   (left/right/center), zero-padded ints, and fixed-point floats.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_format_positional_and_numbered_fields() {
    let out = jit_capture(
        r#"print("{} {}".format("hello", "world"))
print("{0} {1} {0}".format("a", "b"))
"#,
    );
    assert_output(&out, "hello world\na b a\n");
}

#[test]
fn test_format_keyword_fields() {
    let out = jit_capture(
        r#"print("{name} is {age}".format(name="Alice", age=30))
print("{a}+{b}={c}".format(a=1, b=2, c=3))
"#,
    );
    assert_output(&out, "Alice is 30\n1+2=3\n");
}

#[test]
fn test_format_alignment_specifiers() {
    let out = jit_capture(
        r#"print(repr("{:>5}".format("hi")))
print(repr("{:<5}".format("hi")))
print(repr("{:^5}".format("hi")))
"#,
    );
    assert_output(&out, "'   hi'\n'hi   '\n' hi  '\n");
}

#[test]
fn test_format_numeric_specifiers() {
    let out = jit_capture(
        r#"print("{:05d}".format(42))
print("{:.2f}".format(3.14159))
print("{:.4f}".format(2.71828))
"#,
    );
    assert_output(&out, "00042\n3.14\n2.7183\n");
}

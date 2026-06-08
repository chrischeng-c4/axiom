//! Py3.12 conformance tests for `print` keyword arguments (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_print.py — sep/end
//! sections):
//!   custom `sep`, empty `sep`, and custom `end` that suppresses the
//!   trailing newline so the next `print` continues on the same line.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_print_custom_separator() {
    let out = jit_capture(
        r#"print("a", "b", "c", sep="-")
print(1, 2, 3, sep=", ")
"#,
    );
    assert_output(&out, "a-b-c\n1, 2, 3\n");
}

#[test]
fn test_print_empty_separator_concatenates() {
    let out = jit_capture(
        r#"print("x", "y", "z", sep="")
"#,
    );
    assert_output(&out, "xyz\n");
}

#[test]
fn test_print_custom_end_suppresses_newline() {
    let out = jit_capture(
        r#"print("hello", end="")
print(" world")
print("a", end="|")
print("b", end="|")
print("c")
"#,
    );
    assert_output(&out, "hello world\na|b|c\n");
}

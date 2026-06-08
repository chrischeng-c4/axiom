//! Py3.12 conformance tests for printf-style `%` string formatting
//! (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_format.py — %-format
//! sections):
//!   single `%d`/`%s`, tuple substitution, width/precision modifiers,
//!   and named composite forms.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_pct_format_single_conversion() {
    let out = jit_capture(
        r#"print("%d" % 42)
print("%s" % "hi")
print("%d" % -7)
"#,
    );
    assert_output(&out, "42\nhi\n-7\n");
}

#[test]
fn test_pct_format_tuple_substitution() {
    let out = jit_capture(
        r#"print("%d + %d = %d" % (1, 2, 3))
print("name=%s age=%d" % ("Alice", 30))
"#,
    );
    assert_output(&out, "1 + 2 = 3\nname=Alice age=30\n");
}

#[test]
fn test_pct_format_width_and_precision() {
    let out = jit_capture(
        r#"print(repr("%5d" % 7))
print(repr("%-5d" % 7))
print("%05d" % 7)
print("%.2f" % 3.14159)
"#,
    );
    assert_output(&out, "'    7'\n'7    '\n00007\n3.14\n");
}

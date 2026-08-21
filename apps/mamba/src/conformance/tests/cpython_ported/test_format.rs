//! Py3.12 conformance tests for `str.format` (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_format.py — basic
//! format-method coverage):
//!   positional, named (kwarg), indexed (positional reuse).
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_format_positional_single() {
    let out = jit_capture(
        r#"print("hello {}".format("world"))
"#,
    );
    assert_output(&out, "hello world\n");
}

#[test]
fn test_format_positional_multi() {
    let out = jit_capture(
        r#"print("{} + {} = {}".format(1, 2, 3))
"#,
    );
    assert_output(&out, "1 + 2 = 3\n");
}

#[test]
fn test_format_named_kwarg() {
    let out = jit_capture(
        r#"print("{name}={val}".format(name="x", val=42))
"#,
    );
    assert_output(&out, "x=42\n");
}

#[test]
fn test_format_indexed_reuse() {
    let out = jit_capture(
        r#"print("{0} {1} {0}".format("a", "b"))
"#,
    );
    assert_output(&out, "a b a\n");
}

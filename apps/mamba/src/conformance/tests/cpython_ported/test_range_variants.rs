//! Py3.12 conformance tests for `range` argument variants (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_range.py — argument
//! sections):
//!   one-arg `range(stop)`, two-arg `range(start, stop)`, three-arg
//!   `range(start, stop, step)` including negative step, plus `len`
//!   of a range and empty range.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_range_one_and_two_arg_forms() {
    let out = jit_capture(
        r#"print(list(range(5)))
print(list(range(2, 7)))
"#,
    );
    assert_output(&out, "[0, 1, 2, 3, 4]\n[2, 3, 4, 5, 6]\n");
}

#[test]
fn test_range_with_positive_step() {
    let out = jit_capture(
        r#"print(list(range(1, 10, 2)))
print(list(range(0, 20, 5)))
"#,
    );
    assert_output(&out, "[1, 3, 5, 7, 9]\n[0, 5, 10, 15]\n");
}

#[test]
fn test_range_with_negative_step() {
    let out = jit_capture(
        r#"print(list(range(10, 0, -1)))
print(list(range(10, 0, -2)))
"#,
    );
    assert_output(&out, "[10, 9, 8, 7, 6, 5, 4, 3, 2, 1]\n[10, 8, 6, 4, 2]\n");
}

#[test]
fn test_range_len_and_empty() {
    let out = jit_capture(
        r#"print(len(range(100)))
print(list(range(0)))
print(list(range(5, 5)))
"#,
    );
    assert_output(&out, "100\n[]\n[]\n");
}

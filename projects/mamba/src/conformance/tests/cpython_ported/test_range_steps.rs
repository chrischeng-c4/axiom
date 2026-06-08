//! Py3.12 conformance tests for `range` step and bound variants
//! (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_range.py — step
//! and bound sections): forward step, reverse step, empty range
//! (lo==hi), negative-bound range, and reductions over `range`
//! (`len`, `sum`).
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_range_forward_steps() {
    let out = jit_capture(
        r#"print(list(range(10)))
print(list(range(2, 8)))
print(list(range(0, 10, 2)))
"#,
    );
    assert_output(
        &out,
        "[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]\n[2, 3, 4, 5, 6, 7]\n[0, 2, 4, 6, 8]\n",
    );
}

#[test]
fn test_range_reverse_and_empty() {
    let out = jit_capture(
        r#"print(list(range(10, 0, -1)))
print(list(range(10, 0, -2)))
print(list(range(5, 5)))
print(list(range(0, -5, -1)))
"#,
    );
    assert_output(
        &out,
        "[10, 9, 8, 7, 6, 5, 4, 3, 2, 1]\n[10, 8, 6, 4, 2]\n[]\n[0, -1, -2, -3, -4]\n",
    );
}

#[test]
fn test_range_len_and_sum() {
    let out = jit_capture(
        r#"print(len(range(100)))
print(sum(range(1, 11)))
"#,
    );
    assert_output(&out, "100\n55\n");
}

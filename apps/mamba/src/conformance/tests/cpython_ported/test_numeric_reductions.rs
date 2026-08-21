//! Py3.12 conformance tests for numeric reductions on lists
//! (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_builtin.py — sum/min/
//! max/sorted sections):
//!   `sum`, `min`, `max`, average via `sum/len`, and `sorted` over
//!   integer lists.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_sum_min_max_avg() {
    let out = jit_capture(
        r#"xs = [1, 2, 3, 4, 5]
print(sum(xs))
print(min(xs))
print(max(xs))
print(sum(xs) / len(xs))
"#,
    );
    assert_output(&out, "15\n1\n5\n3.0\n");
}

#[test]
fn test_sorted_reverse_and_dedupe() {
    let out = jit_capture(
        r#"xs = [3, 1, 4, 1, 5, 9, 2, 6]
print(sorted(xs))
print(sorted(xs, reverse=True))
print(sorted(set(xs)))
"#,
    );
    assert_output(
        &out,
        "[1, 1, 2, 3, 4, 5, 6, 9]\n[9, 6, 5, 4, 3, 2, 1, 1]\n[1, 2, 3, 4, 5, 6, 9]\n",
    );
}

#[test]
fn test_min_max_with_negatives_and_zero() {
    let out = jit_capture(
        r#"xs = [-5, -1, 0, 3, 7]
print(min(xs))
print(max(xs))
print(sum(xs))
"#,
    );
    assert_output(&out, "-5\n7\n4\n");
}

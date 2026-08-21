//! Py3.12 conformance tests for `range` and `enumerate` (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_range.py and
//! test_enumerate.py — basic sections): one-/two-/three-arg `range`,
//! negative-step `range`, empty `range`, `len(range(...))`, and
//! `enumerate` with and without `start`.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_range_arity_and_step() {
    let out = jit_capture(
        r#"print(list(range(5)))
print(list(range(2, 8)))
print(list(range(0, 10, 2)))
print(list(range(10, 0, -1)))
print(list(range(10, 0, -2)))
"#,
    );
    assert_output(
        &out,
        "[0, 1, 2, 3, 4]\n[2, 3, 4, 5, 6, 7]\n[0, 2, 4, 6, 8]\n[10, 9, 8, 7, 6, 5, 4, 3, 2, 1]\n[10, 8, 6, 4, 2]\n",
    );
}

#[test]
fn test_range_empty_and_len_and_sum() {
    let out = jit_capture(
        r#"print(list(range(0)))
print(list(range(5, 5)))
print(len(range(100)))
print(sum(range(11)))
print(sum(range(1, 101)))
"#,
    );
    assert_output(&out, "[]\n[]\n100\n55\n5050\n");
}

#[test]
fn test_enumerate_default_and_start() {
    let out = jit_capture(
        r#"print(list(enumerate(["a", "b", "c"])))
print(list(enumerate(["x", "y"], start=10)))
print(list(enumerate([])))
print(list(enumerate("abc")))
"#,
    );
    assert_output(
        &out,
        "[(0, 'a'), (1, 'b'), (2, 'c')]\n[(10, 'x'), (11, 'y')]\n[]\n[(0, 'a'), (1, 'b'), (2, 'c')]\n",
    );
}

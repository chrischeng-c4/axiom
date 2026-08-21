//! Py3.12 conformance tests for `list.index`, `list.count`,
//! `list.reverse` and `reversed` (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_list.py — search and
//! reversal sections):
//!   `index` first occurrence, `count` for present/absent values, and
//!   the in-place vs builtin reversal idioms.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_list_index_first_occurrence() {
    let out = jit_capture(
        r#"xs = [1, 2, 3, 2, 1]
print(xs.index(2))
print(xs.index(3))
print([10, 20, 30].index(30))
"#,
    );
    assert_output(&out, "1\n2\n2\n");
}

#[test]
fn test_list_count_present_and_absent() {
    let out = jit_capture(
        r#"xs = [1, 2, 3, 2, 1]
print(xs.count(1))
print(xs.count(2))
print(xs.count(99))
print([].count("x"))
"#,
    );
    assert_output(&out, "2\n2\n0\n0\n");
}

#[test]
fn test_list_reverse_in_place_and_builtin_reversed() {
    let out = jit_capture(
        r#"ys = [1, 2, 3]
ys.reverse()
print(ys)
zs = list(reversed([1, 2, 3]))
print(zs)
print(list(reversed("abc")))
"#,
    );
    assert_output(&out, "[3, 2, 1]\n[3, 2, 1]\n['c', 'b', 'a']\n");
}

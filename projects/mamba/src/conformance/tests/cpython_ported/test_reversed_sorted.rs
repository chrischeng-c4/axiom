//! Py3.12 conformance tests for `reversed` and `sorted` built-ins
//! (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_builtin.py — reversed
//! and sorted sections):
//!   reversed over list/str/range, sorted ascending/descending,
//!   sorted with `key=` callable.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_reversed_list_str_range() {
    let out = jit_capture(
        r#"print(list(reversed([1, 2, 3])))
print(list(reversed("hello")))
print(list(reversed(range(5))))
"#,
    );
    assert_output(
        &out,
        "[3, 2, 1]\n['o', 'l', 'l', 'e', 'h']\n[4, 3, 2, 1, 0]\n",
    );
}

#[test]
fn test_sorted_ints_default() {
    let out = jit_capture(
        r#"print(sorted([3, 1, 4, 1, 5, 9, 2, 6]))
"#,
    );
    assert_output(&out, "[1, 1, 2, 3, 4, 5, 6, 9]\n");
}

#[test]
fn test_sorted_reverse_true() {
    let out = jit_capture(
        r#"print(sorted([3, 1, 4, 1, 5, 9, 2, 6], reverse=True))
"#,
    );
    assert_output(&out, "[9, 6, 5, 4, 3, 2, 1, 1]\n");
}

#[test]
fn test_sorted_with_key_lambda() {
    let out = jit_capture(
        r#"print(sorted([(1, "b"), (3, "a"), (2, "c")], key=lambda p: p[1]))
"#,
    );
    assert_output(&out, "[(3, 'a'), (1, 'b'), (2, 'c')]\n");
}

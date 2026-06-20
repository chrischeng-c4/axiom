//! Py3.12 conformance tests for `list` mutating methods (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_list.py and
//! Lib/test/list_tests.py — method sections):
//!   `append`/`extend`/`insert`, `remove`/`pop`, in-place `sort`, and
//!   the `sorted` builtin.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_list_append_extend_insert() {
    let out = jit_capture(
        r#"xs = [1, 2, 3]
xs.append(4)
print(xs)
xs.extend([5, 6])
print(xs)
xs.insert(0, 0)
print(xs)
"#,
    );
    assert_output(
        &out,
        "[1, 2, 3, 4]\n[1, 2, 3, 4, 5, 6]\n[0, 1, 2, 3, 4, 5, 6]\n",
    );
}

#[test]
fn test_list_remove_and_pop() {
    let out = jit_capture(
        r#"xs = [10, 20, 30, 40]
xs.remove(20)
print(xs)
last = xs.pop()
print(last)
print(xs)
"#,
    );
    assert_output(&out, "[10, 30, 40]\n40\n[10, 30]\n");
}

#[test]
fn test_list_sort_in_place() {
    let out = jit_capture(
        r#"xs = [3, 1, 4, 1, 5, 9, 2, 6]
xs.sort()
print(xs)
"#,
    );
    assert_output(&out, "[1, 1, 2, 3, 4, 5, 6, 9]\n");
}

#[test]
fn test_sorted_builtin_with_reverse() {
    let out = jit_capture(
        r#"print(sorted([3, 1, 2]))
print(sorted([3, 1, 2], reverse=True))
"#,
    );
    assert_output(&out, "[1, 2, 3]\n[3, 2, 1]\n");
}

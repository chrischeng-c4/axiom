//! Py3.12 conformance tests for in-place list mutation (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_list.py — mutation
//! sections): `append`/`extend`/`insert`, `remove`/`pop` (with and
//! without index), `reverse`/`sort`, and `clear`.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_append_extend_insert() {
    let out = jit_capture(
        r#"xs = [1, 2, 3]
xs.append(4)
print(xs)
xs.extend([5, 6])
print(xs)
xs.insert(0, 0)
print(xs)
xs.insert(3, 99)
print(xs)
"#,
    );
    assert_output(
        &out,
        "[1, 2, 3, 4]\n[1, 2, 3, 4, 5, 6]\n[0, 1, 2, 3, 4, 5, 6]\n[0, 1, 2, 99, 3, 4, 5, 6]\n",
    );
}

#[test]
fn test_remove_and_pop_variants() {
    let out = jit_capture(
        r#"xs = [10, 20, 30, 40, 50]
xs.remove(30)
print(xs)
last = xs.pop()
print(last)
print(xs)
first = xs.pop(0)
print(first)
print(xs)
"#,
    );
    assert_output(&out, "[10, 20, 40, 50]\n50\n[10, 20, 40]\n10\n[20, 40]\n");
}

#[test]
fn test_reverse_sort_clear() {
    let out = jit_capture(
        r#"xs = [3, 1, 4, 1, 5, 9, 2, 6]
xs.reverse()
print(xs)
xs.sort()
print(xs)
xs.sort(reverse=True)
print(xs)
xs.clear()
print(xs)
print(len(xs))
"#,
    );
    assert_output(
        &out,
        "[6, 2, 9, 5, 1, 4, 1, 3]\n[1, 1, 2, 3, 4, 5, 6, 9]\n[9, 6, 5, 4, 3, 2, 1, 1]\n[]\n0\n",
    );
}

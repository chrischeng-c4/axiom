//! Py3.12 conformance tests for `sorted`, `reversed`, and
//! in-place `list.sort`/`list.reverse` (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_sort.py and
//! Lib/test/test_iter.py — sort/reverse sections): `sorted` with
//! `reverse=` and `key=`, `reversed` over list and `range`, and
//! in-place mutators on a list.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_sorted_with_reverse_and_key() {
    let out = jit_capture(
        r#"xs = [3, 1, 4, 1, 5, 9, 2, 6]
print(sorted(xs))
print(sorted(xs, reverse=True))

words = ["banana", "apple", "cherry"]
print(sorted(words, key=len))
print(sorted(words, key=len, reverse=True))
"#,
    );
    assert_output(
        &out,
        "[1, 1, 2, 3, 4, 5, 6, 9]\n[9, 6, 5, 4, 3, 2, 1, 1]\n['apple', 'banana', 'cherry']\n['cherry', 'banana', 'apple']\n",
    );
}

#[test]
fn test_reversed_over_list_and_range() {
    let out = jit_capture(
        r#"print(list(reversed([1, 2, 3, 4, 5])))
print(list(reversed(range(5))))
"#,
    );
    assert_output(&out, "[5, 4, 3, 2, 1]\n[4, 3, 2, 1, 0]\n");
}

#[test]
fn test_list_inplace_sort_and_reverse() {
    let out = jit_capture(
        r#"ys = [5, 2, 8, 1]
ys.sort()
print(ys)
ys.sort(reverse=True)
print(ys)

zs = [1, 2, 3, 4]
zs.reverse()
print(zs)
"#,
    );
    assert_output(&out, "[1, 2, 5, 8]\n[8, 5, 2, 1]\n[4, 3, 2, 1]\n");
}

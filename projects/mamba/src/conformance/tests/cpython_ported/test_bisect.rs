//! Py3.12 conformance tests for the `bisect` module (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_bisect.py):
//!   bisect_left, bisect_right, insort, insort_left.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_bisect_left_insertion_point() {
    let out = jit_capture(
        r#"import bisect
print(bisect.bisect_left([1, 3, 5, 7, 9], 4))
"#,
    );
    assert_output(&out, "2\n");
}

#[test]
fn test_bisect_right_insertion_point() {
    let out = jit_capture(
        r#"import bisect
print(bisect.bisect_right([1, 3, 5, 7, 9], 5))
"#,
    );
    assert_output(&out, "3\n");
}

#[test]
fn test_bisect_insort_maintains_order() {
    let out = jit_capture(
        r#"import bisect
xs = [1, 3, 5, 7, 9]
bisect.insort(xs, 4)
print(xs)
"#,
    );
    assert_output(&out, "[1, 3, 4, 5, 7, 9]\n");
}

#[test]
fn test_bisect_insort_left_duplicates() {
    let out = jit_capture(
        r#"import bisect
xs = [1, 2, 3, 4]
bisect.insort_left(xs, 2)
print(xs)
"#,
    );
    assert_output(&out, "[1, 2, 2, 3, 4]\n");
}

#[test]
fn test_bisect_bisect_left_boundaries() {
    let out = jit_capture(
        r#"import bisect
print(bisect.bisect_left([1, 2, 3], 0))
print(bisect.bisect_left([1, 2, 3], 4))
"#,
    );
    assert_output(&out, "0\n3\n");
}

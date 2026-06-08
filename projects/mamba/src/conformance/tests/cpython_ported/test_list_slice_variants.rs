//! Py3.12 conformance tests for `list` slice read variants (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_list.py and
//! Lib/test/list_tests.py — slice sections):
//!   open-ended slices, step slices (including reverse), and negative
//!   indices.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_list_basic_and_open_ended_slices() {
    let out = jit_capture(
        r#"xs = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]
print(xs[2:5])
print(xs[:4])
print(xs[6:])
print(xs[:])
"#,
    );
    assert_output(&out, "[2, 3, 4]\n[0, 1, 2, 3]\n[6, 7, 8, 9]\n[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]\n");
}

#[test]
fn test_list_step_slices_including_reverse() {
    let out = jit_capture(
        r#"xs = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]
print(xs[::2])
print(xs[1::2])
print(xs[::-1])
"#,
    );
    assert_output(&out, "[0, 2, 4, 6, 8]\n[1, 3, 5, 7, 9]\n[9, 8, 7, 6, 5, 4, 3, 2, 1, 0]\n");
}

#[test]
fn test_list_negative_index_slices() {
    let out = jit_capture(
        r#"xs = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]
print(xs[-3:])
print(xs[-5:-2])
print(xs[:-5])
"#,
    );
    assert_output(&out, "[7, 8, 9]\n[5, 6, 7]\n[0, 1, 2, 3, 4]\n");
}

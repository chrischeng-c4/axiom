//! Py3.12 conformance tests for sequence slicing (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_slice.py):
//!   string slice variants, step, negative step (reverse), list slice
//!   with step.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_string_basic_and_open_slices() {
    let out = jit_capture(
        r#"s = "abcdefgh"
print(s[2:5])
print(s[:3])
print(s[5:])
print(s[-3:])
"#,
    );
    assert_output(&out, "cde\nabc\nfgh\nfgh\n");
}

#[test]
fn test_string_step_and_reverse() {
    let out = jit_capture(
        r#"s = "abcdefgh"
print(s[::2])
print(s[::-1])
"#,
    );
    assert_output(&out, "aceg\nhgfedcba\n");
}

#[test]
fn test_list_slice_with_step() {
    let out = jit_capture(
        r#"xs = [0, 1, 2, 3, 4, 5, 6]
print(xs[2:5])
print(xs[::2])
print(xs[::-1])
"#,
    );
    assert_output(&out, "[2, 3, 4]\n[0, 2, 4, 6]\n[6, 5, 4, 3, 2, 1, 0]\n");
}

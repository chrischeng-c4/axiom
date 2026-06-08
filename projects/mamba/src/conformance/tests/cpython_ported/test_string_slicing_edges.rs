//! Py3.12 conformance tests for `str` slicing edge cases
//! (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_str.py — slicing
//! sections): basic [a:b] slices, reverse and step slices,
//! negative indices, out-of-range bounds, and `len()` on a slice.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_str_slice_basic_and_reverse() {
    let out = jit_capture(
        r#"s = "abcdefgh"
print(s[:])
print(s[2:])
print(s[:3])
print(s[2:5])
print(s[::-1])
"#,
    );
    assert_output(
        &out,
        "abcdefgh\ncdefgh\nabc\ncde\nhgfedcba\n",
    );
}

#[test]
fn test_str_slice_step_and_negative() {
    let out = jit_capture(
        r#"s = "abcdefgh"
print(s[::2])
print(s[1::2])
print(s[-3:])
print(s[:-3])
print(s[-5:-2])
"#,
    );
    assert_output(
        &out,
        "aceg\nbdfh\nfgh\nabcde\ndef\n",
    );
}

#[test]
fn test_str_slice_out_of_range_and_len() {
    let out = jit_capture(
        r#"s = "abcdefgh"
print(repr(s[10:]))
print(repr(s[5:2]))
print(len(s[2:5]))
"#,
    );
    assert_output(&out, "''\n''\n3\n");
}

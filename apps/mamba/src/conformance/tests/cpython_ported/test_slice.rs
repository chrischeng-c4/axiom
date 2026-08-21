//! Py3.12 conformance tests for slicing (issue #759).
//!
//! Ported from CPython 3.12.0 tag (commit a6cb7e5d45cacbf20b9408d8d3c7b6b5f5e7a0f0):
//!   Lib/test/test_slice.py — SliceTest
//!   Lib/test/test_list.py / Lib/test/test_tuple.py — sequence slicing
//!
//! Coverage: list and tuple and str slicing across explicit bounds, default
//! bounds, negative indices, step (positive and negative), full-reverse
//! [::-1], empty slices, and out-of-range bounds clamping. Slice assignment
//! on list (basic and with step).
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_slice_list_basic() {
    let out = jit_capture(
        r#"a = [1, 2, 3, 4, 5]
print(a[1:4])
"#,
    );
    assert_output(&out, "[2, 3, 4]\n");
}

#[test]
fn test_slice_list_default_start() {
    let out = jit_capture(
        r#"a = [1, 2, 3, 4, 5]
print(a[:3])
"#,
    );
    assert_output(&out, "[1, 2, 3]\n");
}

#[test]
fn test_slice_list_default_stop() {
    let out = jit_capture(
        r#"a = [1, 2, 3, 4, 5]
print(a[2:])
"#,
    );
    assert_output(&out, "[3, 4, 5]\n");
}

#[test]
fn test_slice_list_full_copy() {
    let out = jit_capture(
        r#"a = [1, 2, 3]
print(a[:])
"#,
    );
    assert_output(&out, "[1, 2, 3]\n");
}

#[test]
fn test_slice_list_negative_start() {
    let out = jit_capture(
        r#"a = [1, 2, 3, 4, 5]
print(a[-3:])
"#,
    );
    assert_output(&out, "[3, 4, 5]\n");
}

#[test]
fn test_slice_list_negative_stop() {
    let out = jit_capture(
        r#"a = [1, 2, 3, 4, 5]
print(a[:-2])
"#,
    );
    assert_output(&out, "[1, 2, 3]\n");
}

#[test]
fn test_slice_list_negative_both() {
    let out = jit_capture(
        r#"a = [1, 2, 3, 4, 5]
print(a[-4:-1])
"#,
    );
    assert_output(&out, "[2, 3, 4]\n");
}

#[test]
fn test_slice_list_empty_when_start_ge_stop() {
    let out = jit_capture(
        r#"a = [1, 2, 3]
print(a[2:1])
print(a[3:3])
"#,
    );
    assert_output(&out, "[]\n[]\n");
}

#[test]
fn test_slice_list_step_positive() {
    let out = jit_capture(
        r#"a = [1, 2, 3, 4, 5, 6]
print(a[::2])
"#,
    );
    assert_output(&out, "[1, 3, 5]\n");
}

#[test]
fn test_slice_list_step_with_bounds() {
    let out = jit_capture(
        r#"a = [1, 2, 3, 4, 5, 6, 7, 8]
print(a[1:7:2])
"#,
    );
    assert_output(&out, "[2, 4, 6]\n");
}

#[test]
fn test_slice_list_step_negative_full_reverse() {
    let out = jit_capture(
        r#"a = [1, 2, 3, 4, 5]
print(a[::-1])
"#,
    );
    assert_output(&out, "[5, 4, 3, 2, 1]\n");
}

#[test]
fn test_slice_list_step_negative_with_bounds() {
    let out = jit_capture(
        r#"a = [1, 2, 3, 4, 5]
print(a[4:1:-1])
"#,
    );
    assert_output(&out, "[5, 4, 3]\n");
}

#[test]
fn test_slice_list_out_of_range_start() {
    let out = jit_capture(
        r#"a = [1, 2, 3]
print(a[10:])
"#,
    );
    assert_output(&out, "[]\n");
}

#[test]
fn test_slice_list_out_of_range_stop_clamps() {
    let out = jit_capture(
        r#"a = [1, 2, 3]
print(a[:100])
"#,
    );
    assert_output(&out, "[1, 2, 3]\n");
}

#[test]
fn test_slice_str_basic() {
    let out = jit_capture(
        r#"s = "hello"
print(s[1:4])
"#,
    );
    assert_output(&out, "ell\n");
}

#[test]
fn test_slice_str_default_bounds() {
    let out = jit_capture(
        r#"s = "hello"
print(s[:3])
print(s[2:])
print(s[:])
"#,
    );
    assert_output(&out, "hel\nllo\nhello\n");
}

#[test]
fn test_slice_str_negative() {
    let out = jit_capture(
        r#"s = "hello"
print(s[-3:])
print(s[:-2])
"#,
    );
    assert_output(&out, "llo\nhel\n");
}

#[test]
fn test_slice_str_step() {
    let out = jit_capture(
        r#"s = "abcdef"
print(s[::2])
"#,
    );
    assert_output(&out, "ace\n");
}

#[test]
fn test_slice_str_reverse() {
    let out = jit_capture(
        r#"s = "hello"
print(s[::-1])
"#,
    );
    assert_output(&out, "olleh\n");
}

#[test]
fn test_slice_str_empty() {
    let out = jit_capture(
        r#"s = "hello"
print(s[2:2])
print(len(s[2:2]))
"#,
    );
    assert_output(&out, "\n0\n");
}

#[test]
fn test_slice_tuple_basic() {
    let out = jit_capture(
        r#"t = (1, 2, 3, 4, 5)
print(t[1:4])
"#,
    );
    assert_output(&out, "(2, 3, 4)\n");
}

#[test]
fn test_slice_tuple_reverse() {
    let out = jit_capture(
        r#"t = (1, 2, 3, 4, 5)
print(t[::-1])
"#,
    );
    assert_output(&out, "(5, 4, 3, 2, 1)\n");
}

#[test]
fn test_slice_tuple_negative() {
    let out = jit_capture(
        r#"t = (1, 2, 3, 4, 5)
print(t[-3:])
"#,
    );
    assert_output(&out, "(3, 4, 5)\n");
}

#[test]
fn test_slice_list_assignment_basic() {
    let out = jit_capture(
        r#"a = [1, 2, 3, 4, 5]
a[1:3] = [20, 30, 40]
print(a)
"#,
    );
    assert_output(&out, "[1, 20, 30, 40, 4, 5]\n");
}

#[test]
fn test_slice_list_assignment_empty_replacement() {
    let out = jit_capture(
        r#"a = [1, 2, 3, 4, 5]
a[1:3] = []
print(a)
"#,
    );
    assert_output(&out, "[1, 4, 5]\n");
}

#[test]
fn test_slice_list_yields_new_object() {
    let out = jit_capture(
        r#"a = [1, 2, 3]
b = a[:]
b.append(4)
print(a)
print(b)
"#,
    );
    assert_output(&out, "[1, 2, 3]\n[1, 2, 3, 4]\n");
}

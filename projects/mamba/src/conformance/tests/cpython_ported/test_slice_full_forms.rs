//! Py3.12 conformance tests for all slice forms on str and list
//! (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_slice.py and
//! test_str.py — slicing sections):
//!   `[:n]`, `[n:]`, `[::k]`, `[::-1]`, `[a:b:k]`, and negative-start
//!   tail slice. Exercises both string and list operand types.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_string_slice_forms() {
    let out = jit_capture(
        r#"s = "hello world"
print(s[:5])
print(s[6:])
print(s[::2])
print(s[::-1])
print(s[1:8:2])
"#,
    );
    assert_output(&out, "hello\nworld\nhlowrd\ndlrow olleh\nel o\n");
}

#[test]
fn test_list_slice_forms() {
    let out = jit_capture(
        r#"xs = list(range(10))
print(xs[2:7])
print(xs[::3])
print(xs[::-1])
print(xs[-3:])
"#,
    );
    assert_output(
        &out,
        "[2, 3, 4, 5, 6]\n[0, 3, 6, 9]\n[9, 8, 7, 6, 5, 4, 3, 2, 1, 0]\n[7, 8, 9]\n",
    );
}

#[test]
fn test_slice_empty_and_clamped_bounds() {
    let out = jit_capture(
        r#"xs = [10, 20, 30, 40]
print(xs[100:])
print(xs[:0])
print(xs[2:2])
print(xs[-100:])
print(xs[:100])
"#,
    );
    assert_output(&out, "[]\n[]\n[]\n[10, 20, 30, 40]\n[10, 20, 30, 40]\n");
}

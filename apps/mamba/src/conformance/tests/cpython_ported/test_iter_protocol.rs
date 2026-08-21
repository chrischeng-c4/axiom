//! Py3.12 conformance tests for the iterator protocol (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_iter.py):
//!   `iter` + `next` exhaust a list, raise `StopIteration`, and the
//!   built-in iterator drains exactly once.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_iter_next_exhausts_list() {
    let out = jit_capture(
        r#"it = iter([1, 2, 3])
print(next(it))
print(next(it))
print(next(it))
"#,
    );
    assert_output(&out, "1\n2\n3\n");
}

#[test]
fn test_iter_next_raises_stop_iteration_when_drained() {
    let out = jit_capture(
        r#"it = iter([1])
print(next(it))
try:
    next(it)
except StopIteration:
    print("stopped")
"#,
    );
    assert_output(&out, "1\nstopped\n");
}

#[test]
fn test_iter_over_string_yields_chars() {
    let out = jit_capture(
        r#"it = iter("abc")
print(next(it))
print(next(it))
print(next(it))
"#,
    );
    assert_output(&out, "a\nb\nc\n");
}

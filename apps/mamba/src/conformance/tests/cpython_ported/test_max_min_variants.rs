//! Py3.12 conformance tests for `max` and `min` variants
//! (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_builtin.py —
//! max/min sections): iterable form, multi-argument form,
//! `default=` on empty iterables, and `key=` callable selector.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_max_min_iterable_and_args() {
    let out = jit_capture(
        r#"print(max([3, 1, 4, 1, 5, 9, 2, 6]))
print(min([3, 1, 4, 1, 5, 9, 2, 6]))
print(max("hello"))
print(min("hello"))
print(max(-3, -1, -2))
print(min(-3, -1, -2))
"#,
    );
    assert_output(&out, "9\n1\no\ne\n-1\n-3\n");
}

#[test]
fn test_max_min_default_on_empty() {
    let out = jit_capture(
        r#"print(max([], default=-1))
print(min([], default=99))
"#,
    );
    assert_output(&out, "-1\n99\n");
}

#[test]
fn test_max_min_with_key_callable() {
    let out = jit_capture(
        r#"print(max([(1, 9), (2, 4), (3, 7)], key=lambda x: x[1]))
print(min([(1, 9), (2, 4), (3, 7)], key=lambda x: x[1]))
"#,
    );
    assert_output(&out, "(1, 9)\n(2, 4)\n");
}

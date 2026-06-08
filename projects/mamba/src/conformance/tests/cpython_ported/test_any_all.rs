//! Py3.12 conformance tests for `any` and `all` builtins (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_builtin.py — any/all
//! sections):
//!   list inputs with mixed truthiness, empty-iterable identity
//!   (`any([])` is False, `all([])` is True), and generator
//!   expressions feeding both.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_any_over_list_inputs() {
    let out = jit_capture(
        r#"print(any([False, False, True]))
print(any([False, False, False]))
print(any([0, 0, 1]))
print(any([0, 0, 0]))
"#,
    );
    assert_output(&out, "True\nFalse\nTrue\nFalse\n");
}

#[test]
fn test_all_over_list_inputs() {
    let out = jit_capture(
        r#"print(all([True, True, True]))
print(all([True, False, True]))
print(all([1, 2, 3]))
print(all([1, 0, 3]))
"#,
    );
    assert_output(&out, "True\nFalse\nTrue\nFalse\n");
}

#[test]
fn test_any_all_on_empty_and_generators() {
    let out = jit_capture(
        r#"print(any([]))
print(all([]))
print(any(x > 5 for x in [1, 2, 3]))
print(any(x > 5 for x in [1, 6, 3]))
print(all(x > 0 for x in [1, 2, 3]))
"#,
    );
    assert_output(&out, "False\nTrue\nFalse\nTrue\nTrue\n");
}

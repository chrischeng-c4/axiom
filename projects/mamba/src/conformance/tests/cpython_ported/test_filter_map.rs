//! Py3.12 conformance tests for `filter` and `map` builtins
//! (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_builtin.py — filter
//! and map sections):
//!   `filter` with a lambda predicate, `filter(None, ...)` keeping
//!   truthy items, and `map` with a single iterable.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_filter_with_lambda_predicate() {
    let out = jit_capture(
        r#"print(list(filter(lambda x: x > 2, [1, 2, 3, 4, 5])))
print(list(filter(lambda x: x % 2 == 0, [1, 2, 3, 4, 5, 6])))
"#,
    );
    assert_output(&out, "[3, 4, 5]\n[2, 4, 6]\n");
}

#[test]
fn test_filter_none_keeps_truthy_items() {
    let out = jit_capture(
        r#"print(list(filter(None, [0, 1, 2, 0, 3])))
print(list(filter(None, ["", "a", "", "b"])))
"#,
    );
    assert_output(&out, "[1, 2, 3]\n['a', 'b']\n");
}

#[test]
fn test_map_with_single_iterable() {
    let out = jit_capture(
        r#"print(list(map(lambda x: x * 2, [1, 2, 3])))
print(list(map(lambda x: x + 10, [1, 2, 3])))
print(list(map(str, [1, 2, 3])))
"#,
    );
    assert_output(&out, "[2, 4, 6]\n[11, 12, 13]\n['1', '2', '3']\n");
}

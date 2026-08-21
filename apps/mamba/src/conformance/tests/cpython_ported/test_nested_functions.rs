//! Py3.12 conformance tests for nested function definitions (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_scope.py — nested-def
//! sections):
//!   inner def captures enclosing locals; nested def returned and called
//!   from caller.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_inner_def_captures_outer_local() {
    let out = jit_capture(
        r#"def make_adder(x):
    def add(y):
        return x + y
    return add
add5 = make_adder(5)
print(add5(3))
print(add5(10))
"#,
    );
    assert_output(&out, "8\n15\n");
}

#[test]
fn test_nested_def_called_inside_outer() {
    let out = jit_capture(
        r#"def outer(n):
    def double(x):
        return x * 2
    return double(n) + 1
print(outer(3))
print(outer(7))
"#,
    );
    assert_output(&out, "7\n15\n");
}

//! Py3.12 conformance tests for integer arithmetic operators
//! (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_int.py — operator
//! sections):
//!   floor division, modulo (with negative operand semantics that
//!   differ from C), `**` exponent including 0-base/0-exp edges, and
//!   unary negation.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_int_floor_division() {
    let out = jit_capture(
        r#"print(10 // 3)
print(-10 // 3)
print(10 // -3)
print(7 // 2)
"#,
    );
    assert_output(&out, "3\n-4\n-4\n3\n");
}

#[test]
fn test_int_modulo_with_negative_operand() {
    let out = jit_capture(
        r#"print(10 % 3)
print(-10 % 3)
print(10 % -3)
print(7 % 2)
"#,
    );
    assert_output(&out, "1\n2\n-2\n1\n");
}

#[test]
fn test_int_exponent_operator() {
    let out = jit_capture(
        r#"print(2 ** 10)
print(2 ** 0)
print(0 ** 5)
print(3 ** 3)
"#,
    );
    assert_output(&out, "1024\n1\n0\n27\n");
}

#[test]
fn test_int_unary_negation() {
    let out = jit_capture(
        r#"print(-(-5))
print(-0)
print(-(7))
print(-(-(-3)))
"#,
    );
    assert_output(&out, "5\n0\n-7\n-3\n");
}

//! Py3.12 conformance tests for arithmetic built-ins (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_builtin.py — arithmetic
//! sections):
//!   `divmod`, `abs`, `pow`, `min` and `max` over both varargs and a
//!   single iterable.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_divmod_returns_quotient_remainder() {
    let out = jit_capture(
        r#"print(divmod(10, 3))
print(divmod(20, 4))
print(divmod(7, 2))
"#,
    );
    assert_output(&out, "(3, 1)\n(5, 0)\n(3, 1)\n");
}

#[test]
fn test_abs_on_int_and_float() {
    let out = jit_capture(
        r#"print(abs(-5))
print(abs(5))
print(abs(-3.5))
print(abs(0))
"#,
    );
    assert_output(&out, "5\n5\n3.5\n0\n");
}

#[test]
fn test_pow_two_argument_form() {
    let out = jit_capture(
        r#"print(pow(2, 10))
print(pow(3, 4))
print(pow(5, 0))
"#,
    );
    assert_output(&out, "1024\n81\n1\n");
}

#[test]
fn test_min_max_varargs_and_iterable() {
    let out = jit_capture(
        r#"print(min(3, 1, 4))
print(max(3, 1, 4))
print(min([5, 2, 8]))
print(max([5, 2, 8]))
"#,
    );
    assert_output(&out, "1\n4\n2\n8\n");
}

//! Py3.12 conformance tests for `abs`/`divmod`/`pow` (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_builtin.py — numeric
//! builtin sections): `abs` over int and float, `divmod` over positive
//! and negative dividends, two-argument `pow`, and three-argument
//! `pow` (modular exponentiation).
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_abs_int_and_float() {
    let out = jit_capture(
        r#"print(abs(5))
print(abs(-3))
print(abs(0))
print(abs(-3.14))
print(abs(2.5))
"#,
    );
    assert_output(&out, "5\n3\n0\n3.14\n2.5\n");
}

#[test]
fn test_divmod_positive_and_negative() {
    let out = jit_capture(
        r#"print(divmod(17, 5))
print(divmod(-17, 5))
print(divmod(20, 4))
print(divmod(0, 7))
"#,
    );
    assert_output(&out, "(3, 2)\n(-4, 3)\n(5, 0)\n(0, 0)\n");
}

#[test]
fn test_pow_two_and_three_args() {
    let out = jit_capture(
        r#"print(pow(2, 10))
print(pow(3, 4))
print(pow(2, 8, 100))
print(pow(7, 3, 13))
print(pow(5, 0))
"#,
    );
    assert_output(&out, "1024\n81\n56\n5\n1\n");
}

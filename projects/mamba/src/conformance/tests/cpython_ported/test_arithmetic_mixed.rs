//! Py3.12 conformance tests for mixed-type arithmetic and division
//! semantics (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_int.py and
//! test_float.py — arithmetic sections): true vs floor division,
//! modulo with sign, exponentiation across int/float, and the unary
//! sign operators with `abs`.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_true_vs_floor_division_and_mod() {
    let out = jit_capture(
        r#"print(10 / 3)
print(10 // 3)
print(10 % 3)
print(-10 // 3)
print(-10 % 3)
print(10 / 5)
print(10 // 5)
"#,
    );
    assert_output(&out, "3.3333333333333335\n3\n1\n-4\n2\n2.0\n2\n");
}

#[test]
fn test_pow_int_and_float() {
    let out = jit_capture(
        r#"print(2 ** 10)
print(3 ** 4)
print(2 ** 0)
print(1.5 ** 2)
print(2.0 ** 3)
print(4 ** 0.5)
"#,
    );
    assert_output(&out, "1024\n81\n1\n2.25\n8.0\n2.0\n");
}

#[test]
fn test_unary_sign_and_abs() {
    let out = jit_capture(
        r#"a = 10
b = -3.5
print(-a)
print(+a)
print(abs(-a))
print(abs(b))
print(-(-a))
print(+(-a))
"#,
    );
    assert_output(&out, "-10\n10\n10\n3.5\n10\n-10\n");
}

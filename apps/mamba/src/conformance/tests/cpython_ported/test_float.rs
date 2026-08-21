//! Py3.12 conformance tests for float (issue #759).
//!
//! Ported from CPython 3.12.0 tag (commit a6cb7e5d45cacbf20b9408d8d3c7b6b5f5e7a0f0):
//!   Lib/test/test_float.py — GeneralFloatCases, FormatTestCase
//!
//! Coverage: arithmetic (+ - * / // % **), comparison, constructor from
//! str/int, abs, bool, str, round, int conversion, mixed-type arithmetic.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_float_addition() {
    let out = jit_capture(
        r#"print(1.5 + 2.5)
print(-1.0 + 1.0)
print(0.0 + 0.0)
"#,
    );
    assert_output(&out, "4.0\n0.0\n0.0\n");
}

#[test]
fn test_float_subtraction() {
    let out = jit_capture(
        r#"print(5.0 - 2.0)
print(2.0 - 5.0)
"#,
    );
    assert_output(&out, "3.0\n-3.0\n");
}

#[test]
fn test_float_multiplication() {
    let out = jit_capture(
        r#"print(2.5 * 4.0)
print(0.0 * 100.0)
"#,
    );
    assert_output(&out, "10.0\n0.0\n");
}

#[test]
fn test_float_true_division() {
    let out = jit_capture(
        r#"print(10.0 / 4.0)
print(1.0 / 2.0)
"#,
    );
    assert_output(&out, "2.5\n0.5\n");
}

#[test]
fn test_float_int_division_returns_float() {
    let out = jit_capture(
        r#"print(10 / 4)
"#,
    );
    assert_output(&out, "2.5\n");
}

#[test]
fn test_float_floor_division() {
    let out = jit_capture(
        r#"print(7.5 // 2.0)
print(-7.5 // 2.0)
"#,
    );
    assert_output(&out, "3.0\n-4.0\n");
}

#[test]
fn test_float_power() {
    let out = jit_capture(
        r#"print(2.0 ** 3.0)
print(4.0 ** 0.5)
"#,
    );
    assert_output(&out, "8.0\n2.0\n");
}

#[test]
fn test_float_less_than() {
    let out = jit_capture(
        r#"print(1.0 < 2.0)
print(2.0 < 1.0)
print(2.0 < 2.0)
"#,
    );
    assert_output(&out, "True\nFalse\nFalse\n");
}

#[test]
fn test_float_greater_than() {
    let out = jit_capture(
        r#"print(2.0 > 1.0)
print(1.0 > 2.0)
"#,
    );
    assert_output(&out, "True\nFalse\n");
}

#[test]
fn test_float_equality() {
    let out = jit_capture(
        r#"print(1.5 == 1.5)
print(1.5 == 2.5)
"#,
    );
    assert_output(&out, "True\nFalse\n");
}

#[test]
fn test_float_constructor_from_string() {
    let out = jit_capture(
        r#"print(float("3.14"))
print(float("-1.5"))
print(float("0"))
"#,
    );
    assert_output(&out, "3.14\n-1.5\n0.0\n");
}

#[test]
fn test_float_constructor_from_int() {
    let out = jit_capture(
        r#"print(float(42))
print(float(-7))
print(float(0))
"#,
    );
    assert_output(&out, "42.0\n-7.0\n0.0\n");
}

#[test]
fn test_float_default_is_zero() {
    let out = jit_capture(
        r#"print(float())
"#,
    );
    assert_output(&out, "0.0\n");
}

#[test]
fn test_float_abs_positive() {
    let out = jit_capture(
        r#"print(abs(3.14))
"#,
    );
    assert_output(&out, "3.14\n");
}

#[test]
fn test_float_abs_negative() {
    let out = jit_capture(
        r#"print(abs(-3.14))
"#,
    );
    assert_output(&out, "3.14\n");
}

#[test]
fn test_float_bool_zero_is_false() {
    let out = jit_capture(
        r#"print(bool(0.0))
print(bool(-0.0))
"#,
    );
    assert_output(&out, "False\nFalse\n");
}

#[test]
fn test_float_bool_nonzero_is_true() {
    let out = jit_capture(
        r#"print(bool(1.0))
print(bool(-0.5))
"#,
    );
    assert_output(&out, "True\nTrue\n");
}

#[test]
fn test_float_int_conversion_truncates() {
    let out = jit_capture(
        r#"print(int(3.7))
print(int(-3.7))
print(int(0.5))
"#,
    );
    assert_output(&out, "3\n-3\n0\n");
}

#[test]
fn test_float_str_conversion_simple() {
    let out = jit_capture(
        r#"print(str(1.5))
print(str(-2.0))
"#,
    );
    assert_output(&out, "1.5\n-2.0\n");
}

#[test]
fn test_float_round_default() {
    let out = jit_capture(
        r#"print(round(3.7))
print(round(3.4))
print(round(-3.7))
"#,
    );
    assert_output(&out, "4\n3\n-4\n");
}

#[test]
fn test_float_mixed_int_float_arithmetic() {
    let out = jit_capture(
        r#"print(2 + 3.0)
print(3.0 + 2)
print(5 * 2.5)
"#,
    );
    assert_output(&out, "5.0\n5.0\n12.5\n");
}

#[test]
fn test_float_unary_negation() {
    let out = jit_capture(
        r#"x = 3.14
print(-x)
print(-(-x))
"#,
    );
    assert_output(&out, "-3.14\n3.14\n");
}

#[test]
fn test_float_augmented_add() {
    let out = jit_capture(
        r#"x = 1.0
x += 0.5
print(x)
"#,
    );
    assert_output(&out, "1.5\n");
}

#[test]
fn test_float_augmented_mul() {
    let out = jit_capture(
        r#"x = 2.0
x *= 3.0
print(x)
"#,
    );
    assert_output(&out, "6.0\n");
}

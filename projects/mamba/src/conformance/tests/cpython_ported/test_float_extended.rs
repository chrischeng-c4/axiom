//! Py3.12 conformance tests for float — extended coverage (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_float.py):
//!   GeneralFloatCases (string parsing for inf/nan/scientific notation,
//!   is_integer, repr, divmod, mod), FormatTestCase (str of -0.0, inf).
//!
//! Complements `test_float.rs` (basic arithmetic) with text-form
//! parsing/formatting and a handful of standard-library float methods.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_float_str_inf() {
    let out = jit_capture(r#"print(float("inf"))"#);
    assert_output(&out, "inf\n");
}

#[test]
fn test_float_str_negative_inf() {
    let out = jit_capture(r#"print(float("-inf"))"#);
    assert_output(&out, "-inf\n");
}

#[test]
fn test_float_str_nan() {
    let out = jit_capture(r#"print(float("nan"))"#);
    assert_output(&out, "nan\n");
}

#[test]
fn test_float_str_infinity_long() {
    let out = jit_capture(r#"print(float("Infinity"))"#);
    assert_output(&out, "inf\n");
}

#[test]
fn test_float_str_scientific_positive() {
    let out = jit_capture(r#"print(float("1e10"))"#);
    assert_output(&out, "10000000000.0\n");
}

#[test]
fn test_float_str_scientific_negative_exp() {
    let out = jit_capture(r#"print(float("1.5e-3"))"#);
    assert_output(&out, "0.0015\n");
}

#[test]
fn test_float_str_negative_zero() {
    let out = jit_capture(r#"print(str(-0.0))"#);
    assert_output(&out, "-0.0\n");
}

#[test]
fn test_float_is_integer_true() {
    let out = jit_capture(r#"print((3.0).is_integer())
print((0.0).is_integer())
print((-7.0).is_integer())
"#);
    assert_output(&out, "True\nTrue\nTrue\n");
}

#[test]
fn test_float_is_integer_false() {
    let out = jit_capture(r#"print((3.5).is_integer())
print((0.1).is_integer())
"#);
    assert_output(&out, "False\nFalse\n");
}

#[test]
fn test_float_modulo() {
    let out = jit_capture(r#"print(7.5 % 2.0)
print(-7.5 % 2.0)
"#);
    assert_output(&out, "1.5\n0.5\n");
}

#[test]
fn test_float_min_of_floats() {
    let out = jit_capture(r#"print(min(1.5, 2.0, 0.5))"#);
    assert_output(&out, "0.5\n");
}

#[test]
fn test_float_max_of_floats() {
    let out = jit_capture(r#"print(max(1.5, 2.0, 0.5))"#);
    assert_output(&out, "2.0\n");
}

#[test]
fn test_float_sum_of_floats() {
    let out = jit_capture(r#"print(sum([1.0, 2.5, 0.5]))"#);
    assert_output(&out, "4.0\n");
}

#[test]
fn test_float_inf_arithmetic() {
    let out = jit_capture(r#"x = float("inf")
print(x + 1.0)
print(x * 2.0)
print(-x)
"#);
    assert_output(&out, "inf\ninf\n-inf\n");
}

#[test]
fn test_float_repr_simple() {
    let out = jit_capture(r#"print(repr(1.5))
print(repr(-2.0))
"#);
    assert_output(&out, "1.5\n-2.0\n");
}

#[test]
fn test_float_comparison_inf() {
    let out = jit_capture(r#"inf = float("inf")
print(inf > 1.0)
print(inf > 1e308)
print(-inf < -1.0)
"#);
    assert_output(&out, "True\nTrue\nTrue\n");
}

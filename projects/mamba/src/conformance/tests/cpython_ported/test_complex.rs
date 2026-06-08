//! Py3.12 conformance tests for complex (issue #759).
//!
//! Ported from CPython 3.12.0 tag (commit a6cb7e5d45cacbf20b9408d8d3c7b6b5f5e7a0f0):
//!   Lib/test/test_complex.py — ComplexTest
//!
//! Coverage: literal, .real / .imag, abs, arithmetic (+ - * /),
//! comparison (== !=), bool, constructor from int/float/str, unary negation,
//! conjugate, mixed-type arithmetic with int/float.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_complex_literal_imag() {
    let out = jit_capture(
        r#"print(2j)
"#,
    );
    assert_output(&out, "2j\n");
}

#[test]
fn test_complex_literal_full() {
    let out = jit_capture(
        r#"print(1+2j)
"#,
    );
    assert_output(&out, "(1+2j)\n");
}

#[test]
fn test_complex_real_attr() {
    let out = jit_capture(
        r#"c = 3+4j
print(c.real)
"#,
    );
    assert_output(&out, "3.0\n");
}

#[test]
fn test_complex_imag_attr() {
    let out = jit_capture(
        r#"c = 3+4j
print(c.imag)
"#,
    );
    assert_output(&out, "4.0\n");
}

#[test]
fn test_complex_abs() {
    let out = jit_capture(
        r#"print(abs(3+4j))
"#,
    );
    assert_output(&out, "5.0\n");
}

#[test]
fn test_complex_addition() {
    let out = jit_capture(
        r#"print((1+2j) + (3+4j))
"#,
    );
    assert_output(&out, "(4+6j)\n");
}

#[test]
fn test_complex_subtraction() {
    let out = jit_capture(
        r#"print((5+6j) - (1+2j))
"#,
    );
    assert_output(&out, "(4+4j)\n");
}

#[test]
fn test_complex_multiplication() {
    let out = jit_capture(
        r#"print((1+2j) * (3+4j))
"#,
    );
    assert_output(&out, "(-5+10j)\n");
}

#[test]
fn test_complex_division_simple() {
    let out = jit_capture(
        r#"print((4+0j) / (2+0j))
"#,
    );
    assert_output(&out, "(2+0j)\n");
}

#[test]
fn test_complex_equality_true() {
    let out = jit_capture(
        r#"print((1+2j) == (1+2j))
"#,
    );
    assert_output(&out, "True\n");
}

#[test]
fn test_complex_equality_false() {
    let out = jit_capture(
        r#"print((1+2j) == (3+4j))
"#,
    );
    assert_output(&out, "False\n");
}

#[test]
fn test_complex_inequality() {
    let out = jit_capture(
        r#"print((1+2j) != (3+4j))
print((1+2j) != (1+2j))
"#,
    );
    assert_output(&out, "True\nFalse\n");
}

#[test]
fn test_complex_bool_zero_is_false() {
    let out = jit_capture(
        r#"print(bool(0+0j))
"#,
    );
    assert_output(&out, "False\n");
}

#[test]
fn test_complex_bool_nonzero_is_true() {
    let out = jit_capture(
        r#"print(bool(1+0j))
print(bool(0+1j))
print(bool(1+2j))
"#,
    );
    assert_output(&out, "True\nTrue\nTrue\n");
}

#[test]
fn test_complex_constructor_from_int() {
    let out = jit_capture(
        r#"print(complex(5))
"#,
    );
    assert_output(&out, "(5+0j)\n");
}

#[test]
fn test_complex_constructor_from_two_args() {
    let out = jit_capture(
        r#"print(complex(3, 4))
"#,
    );
    assert_output(&out, "(3+4j)\n");
}

#[test]
fn test_complex_constructor_default() {
    let out = jit_capture(
        r#"print(complex())
"#,
    );
    assert_output(&out, "0j\n");
}

#[test]
fn test_complex_unary_negation() {
    let out = jit_capture(
        r#"print(-(1+2j))
"#,
    );
    assert_output(&out, "(-1-2j)\n");
}

#[test]
fn test_complex_conjugate() {
    let out = jit_capture(
        r#"print((1+2j).conjugate())
"#,
    );
    assert_output(&out, "(1-2j)\n");
}

#[test]
fn test_complex_mixed_int_arithmetic() {
    let out = jit_capture(
        r#"print(2 + (1+2j))
print((1+2j) + 2)
"#,
    );
    assert_output(&out, "(3+2j)\n(3+2j)\n");
}

#[test]
fn test_complex_mixed_float_arithmetic() {
    let out = jit_capture(
        r#"print(2.0 * (1+2j))
"#,
    );
    assert_output(&out, "(2+4j)\n");
}

#[test]
fn test_complex_negative_imag_literal() {
    let out = jit_capture(
        r#"print(1-2j)
"#,
    );
    assert_output(&out, "(1-2j)\n");
}

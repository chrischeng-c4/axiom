//! Ported from Lib/test/test_int_ported.py
//! Integration tests: builtins/numeric.rs

use super::super::harness::*;

/// Ported from `Lib/test/test_int_ported.py`.
#[test]
fn test_float_is_integer_true() {
    let out = jit_capture(
        r#"print((3.0).is_integer())
print((0.0).is_integer())
print((-7.0).is_integer())
"#,
    );
    assert_output(&out, "True\nTrue\nTrue\n");
}

/// Ported from `Lib/test/test_int_ported.py`.
#[test]
fn test_float_is_integer_false() {
    let out = jit_capture(
        r#"print((3.5).is_integer())
print((0.1).is_integer())
"#,
    );
    assert_output(&out, "False\nFalse\n");
}

/// Ported from `Lib/test/test_int_ported.py`.
#[test]
fn test_float_modulo() {
    let out = jit_capture(
        r#"print(7.5 % 2.0)
print(-7.5 % 2.0)
"#,
    );
    assert_output(&out, "1.5\n0.5\n");
}

/// Ported from `Lib/test/test_int_ported.py`.
#[test]
fn test_float_min_of_floats() {
    let out = jit_capture(r#"print(min(1.5, 2.0, 0.5))"#);
    assert_output(&out, "0.5\n");
}

/// Ported from `Lib/test/test_int_ported.py`.
#[test]
fn test_float_max_of_floats() {
    let out = jit_capture(r#"print(max(1.5, 2.0, 0.5))"#);
    assert_output(&out, "2.0\n");
}

/// Ported from `Lib/test/test_int_ported.py`.
#[test]
fn test_float_sum_of_floats() {
    let out = jit_capture(r#"print(sum([1.0, 2.5, 0.5]))"#);
    assert_output(&out, "4.0\n");
}

/// Ported from `Lib/test/test_int_ported.py`.
#[test]
fn test_float_inf_arithmetic() {
    let out = jit_capture(
        r#"x = float("inf")
print(x + 1.0)
print(x * 2.0)
print(-x)
"#,
    );
    assert_output(&out, "inf\ninf\n-inf\n");
}

/// Ported from `Lib/test/test_int_ported.py`.
#[test]
fn test_float_repr_simple() {
    let out = jit_capture(
        r#"print(repr(1.5))
print(repr(-2.0))
"#,
    );
    assert_output(&out, "1.5\n-2.0\n");
}

/// Ported from `Lib/test/test_int_ported.py`.
#[test]
fn test_float_comparison_inf() {
    let out = jit_capture(
        r#"inf = float("inf")
print(inf > 1.0)
print(inf > 1e308)
print(-inf < -1.0)
"#,
    );
    assert_output(&out, "True\nTrue\nTrue\n");
}

/// Ported from `Lib/test/test_int_ported.py`.
#[test]
fn test_bin_hex_oct_with_prefixes() {
    let out = jit_capture(
        r#"print(bin(10))
print(hex(255))
print(oct(8))
"#,
    );
    assert_output(&out, "0b1010\n0xff\n0o10\n");
}

/// Ported from `Lib/test/test_int_ported.py`.
#[test]
fn test_bin_hex_oct_on_zero() {
    let out = jit_capture(
        r#"print(bin(0))
print(hex(0))
print(oct(0))
"#,
    );
    assert_output(&out, "0b0\n0x0\n0o0\n");
}

/// Ported from `Lib/test/test_int_ported.py`.
#[test]
fn test_float_arithmetic_basics() {
    let out = jit_capture(
        r#"x = 3.14
print(x * 2)
print(x / 2)
print(x // 1)
print(2.5 * 4)
print(10.0 / 4)
"#,
    );
    assert_output(&out, "6.28\n1.57\n3.0\n10.0\n2.5\n");
}

/// Ported from `Lib/test/test_int_ported.py`.
#[test]
fn test_round_with_ndigits() {
    let out = jit_capture(
        r#"print(round(3.14159, 1))
print(round(2.567, 2))
print(round(1.5))
print(round(2.5))
print(round(0.1234567, 3))
"#,
    );
    assert_output(&out, "3.1\n2.57\n2\n2\n0.123\n");
}

/// Ported from `Lib/test/test_int_ported.py`.
#[test]
fn test_complex_literal_imag() {
    let out = jit_capture(
        r#"print(2j)
"#,
    );
    assert_output(&out, "2j\n");
}

/// Ported from `Lib/test/test_int_ported.py`.
#[test]
fn test_complex_literal_full() {
    let out = jit_capture(
        r#"print(1+2j)
"#,
    );
    assert_output(&out, "(1+2j)\n");
}

/// Ported from `Lib/test/test_int_ported.py`.
#[test]
fn test_complex_real_attr() {
    let out = jit_capture(
        r#"c = 3+4j
print(c.real)
"#,
    );
    assert_output(&out, "3.0\n");
}

/// Ported from `Lib/test/test_int_ported.py`.
#[test]
fn test_complex_imag_attr() {
    let out = jit_capture(
        r#"c = 3+4j
print(c.imag)
"#,
    );
    assert_output(&out, "4.0\n");
}

/// Ported from `Lib/test/test_int_ported.py`.
#[test]
fn test_complex_abs() {
    let out = jit_capture(
        r#"print(abs(3+4j))
"#,
    );
    assert_output(&out, "5.0\n");
}

/// Ported from `Lib/test/test_int_ported.py`.
#[test]
fn test_complex_addition() {
    let out = jit_capture(
        r#"print((1+2j) + (3+4j))
"#,
    );
    assert_output(&out, "(4+6j)\n");
}

/// Ported from `Lib/test/test_int_ported.py`.
#[test]
fn test_complex_subtraction() {
    let out = jit_capture(
        r#"print((5+6j) - (1+2j))
"#,
    );
    assert_output(&out, "(4+4j)\n");
}

/// Ported from `Lib/test/test_int_ported.py`.
#[test]
fn test_complex_multiplication() {
    let out = jit_capture(
        r#"print((1+2j) * (3+4j))
"#,
    );
    assert_output(&out, "(-5+10j)\n");
}

/// Ported from `Lib/test/test_int_ported.py`.
#[test]
fn test_complex_division_simple() {
    let out = jit_capture(
        r#"print((4+0j) / (2+0j))
"#,
    );
    assert_output(&out, "(2+0j)\n");
}

/// Ported from `Lib/test/test_int_ported.py`.
#[test]
fn test_complex_equality_true() {
    let out = jit_capture(
        r#"print((1+2j) == (1+2j))
"#,
    );
    assert_output(&out, "True\n");
}

/// Ported from `Lib/test/test_int_ported.py`.
#[test]
fn test_complex_equality_false() {
    let out = jit_capture(
        r#"print((1+2j) == (3+4j))
"#,
    );
    assert_output(&out, "False\n");
}

/// Ported from `Lib/test/test_int_ported.py`.
#[test]
fn test_complex_inequality() {
    let out = jit_capture(
        r#"print((1+2j) != (3+4j))
print((1+2j) != (1+2j))
"#,
    );
    assert_output(&out, "True\nFalse\n");
}

/// Ported from `Lib/test/test_int_ported.py`.
#[test]
fn test_complex_bool_zero_is_false() {
    let out = jit_capture(
        r#"print(bool(0+0j))
"#,
    );
    assert_output(&out, "False\n");
}

/// Ported from `Lib/test/test_int_ported.py`.
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

/// Ported from `Lib/test/test_int_ported.py`.
#[test]
fn test_complex_constructor_from_int() {
    let out = jit_capture(
        r#"print(complex(5))
"#,
    );
    assert_output(&out, "(5+0j)\n");
}

/// Ported from `Lib/test/test_int_ported.py`.
#[test]
fn test_complex_constructor_from_two_args() {
    let out = jit_capture(
        r#"print(complex(3, 4))
"#,
    );
    assert_output(&out, "(3+4j)\n");
}

/// Ported from `Lib/test/test_int_ported.py`.
#[test]
fn test_complex_constructor_default() {
    let out = jit_capture(
        r#"print(complex())
"#,
    );
    assert_output(&out, "0j\n");
}

/// Ported from `Lib/test/test_int_ported.py`.
#[test]
fn test_complex_unary_negation() {
    let out = jit_capture(
        r#"print(-(1+2j))
"#,
    );
    assert_output(&out, "(-1-2j)\n");
}

/// Ported from `Lib/test/test_int_ported.py`.
#[test]
fn test_complex_conjugate() {
    let out = jit_capture(
        r#"print((1+2j).conjugate())
"#,
    );
    assert_output(&out, "(1-2j)\n");
}

/// Ported from `Lib/test/test_int_ported.py`.
#[test]
fn test_complex_mixed_int_arithmetic() {
    let out = jit_capture(
        r#"print(2 + (1+2j))
print((1+2j) + 2)
"#,
    );
    assert_output(&out, "(3+2j)\n(3+2j)\n");
}

/// Ported from `Lib/test/test_int_ported.py`.
#[test]
fn test_complex_mixed_float_arithmetic() {
    let out = jit_capture(
        r#"print(2.0 * (1+2j))
"#,
    );
    assert_output(&out, "(2+4j)\n");
}

/// Ported from `Lib/test/test_int_ported.py`.
#[test]
fn test_complex_negative_imag_literal() {
    let out = jit_capture(
        r#"print(1-2j)
"#,
    );
    assert_output(&out, "(1-2j)\n");
}

/// Ported from `Lib/test/test_int_ported.py`.
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

/// Ported from `Lib/test/test_int_ported.py`.
#[test]
fn test_float_subtraction() {
    let out = jit_capture(
        r#"print(5.0 - 2.0)
print(2.0 - 5.0)
"#,
    );
    assert_output(&out, "3.0\n-3.0\n");
}

/// Ported from `Lib/test/test_int_ported.py`.
#[test]
fn test_float_multiplication() {
    let out = jit_capture(
        r#"print(2.5 * 4.0)
print(0.0 * 100.0)
"#,
    );
    assert_output(&out, "10.0\n0.0\n");
}

/// Ported from `Lib/test/test_int_ported.py`.
#[test]
fn test_float_true_division() {
    let out = jit_capture(
        r#"print(10.0 / 4.0)
print(1.0 / 2.0)
"#,
    );
    assert_output(&out, "2.5\n0.5\n");
}

/// Ported from `Lib/test/test_int_ported.py`.
#[test]
fn test_float_int_division_returns_float() {
    let out = jit_capture(
        r#"print(10 / 4)
"#,
    );
    assert_output(&out, "2.5\n");
}

/// Ported from `Lib/test/test_int_ported.py`.
#[test]
fn test_float_floor_division() {
    let out = jit_capture(
        r#"print(7.5 // 2.0)
print(-7.5 // 2.0)
"#,
    );
    assert_output(&out, "3.0\n-4.0\n");
}

/// Ported from `Lib/test/test_int_ported.py`.
#[test]
fn test_float_power() {
    let out = jit_capture(
        r#"print(2.0 ** 3.0)
print(4.0 ** 0.5)
"#,
    );
    assert_output(&out, "8.0\n2.0\n");
}

/// Ported from `Lib/test/test_int_ported.py`.
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

/// Ported from `Lib/test/test_int_ported.py`.
#[test]
fn test_float_greater_than() {
    let out = jit_capture(
        r#"print(2.0 > 1.0)
print(1.0 > 2.0)
"#,
    );
    assert_output(&out, "True\nFalse\n");
}

/// Ported from `Lib/test/test_int_ported.py`.
#[test]
fn test_float_equality() {
    let out = jit_capture(
        r#"print(1.5 == 1.5)
print(1.5 == 2.5)
"#,
    );
    assert_output(&out, "True\nFalse\n");
}

/// Ported from `Lib/test/test_int_ported.py`.
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

/// Ported from `Lib/test/test_int_ported.py`.
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

/// Ported from `Lib/test/test_int_ported.py`.
#[test]
fn test_float_default_is_zero() {
    let out = jit_capture(
        r#"print(float())
"#,
    );
    assert_output(&out, "0.0\n");
}

/// Ported from `Lib/test/test_int_ported.py`.
#[test]
fn test_float_abs_positive() {
    let out = jit_capture(
        r#"print(abs(3.14))
"#,
    );
    assert_output(&out, "3.14\n");
}

/// Ported from `Lib/test/test_int_ported.py`.
#[test]
fn test_float_abs_negative() {
    let out = jit_capture(
        r#"print(abs(-3.14))
"#,
    );
    assert_output(&out, "3.14\n");
}

/// Ported from `Lib/test/test_int_ported.py`.
#[test]
fn test_float_bool_zero_is_false() {
    let out = jit_capture(
        r#"print(bool(0.0))
print(bool(-0.0))
"#,
    );
    assert_output(&out, "False\nFalse\n");
}

/// Ported from `Lib/test/test_int_ported.py`.
#[test]
fn test_float_bool_nonzero_is_true() {
    let out = jit_capture(
        r#"print(bool(1.0))
print(bool(-0.5))
"#,
    );
    assert_output(&out, "True\nTrue\n");
}

/// Ported from `Lib/test/test_int_ported.py`.
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

/// Ported from `Lib/test/test_int_ported.py`.
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

/// Ported from `Lib/test/test_int_ported.py`.
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

/// Ported from `Lib/test/test_int_ported.py`.
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

/// Ported from `Lib/test/test_int_ported.py`.
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

/// Ported from `Lib/test/test_int_ported.py`.
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

/// Ported from `Lib/test/test_int_ported.py`.
#[test]
fn test_int_addition() {
    let out = jit_capture(
        r#"print(2 + 3)
print(-5 + 7)
print(0 + 0)
"#,
    );
    assert_output(&out, "5\n2\n0\n");
}

/// Ported from `Lib/test/test_int_ported.py`.
#[test]
fn test_int_subtraction() {
    let out = jit_capture(
        r#"print(10 - 3)
print(3 - 10)
print(0 - 0)
"#,
    );
    assert_output(&out, "7\n-7\n0\n");
}

/// Ported from `Lib/test/test_int_ported.py`.
#[test]
fn test_int_multiplication() {
    let out = jit_capture(
        r#"print(4 * 5)
print(-3 * 6)
print(0 * 100)
"#,
    );
    assert_output(&out, "20\n-18\n0\n");
}

/// Ported from `Lib/test/test_int_ported.py`.
#[test]
fn test_int_floor_division() {
    let out = jit_capture(
        r#"print(17 // 5)
print(20 // 4)
print(-17 // 5)
"#,
    );
    assert_output(&out, "3\n5\n-4\n");
}

/// Ported from `Lib/test/test_int_ported.py`.
#[test]
fn test_int_modulo() {
    let out = jit_capture(
        r#"print(17 % 5)
print(20 % 4)
print(10 % 3)
"#,
    );
    assert_output(&out, "2\n0\n1\n");
}

/// Ported from `Lib/test/test_int_ported.py`.
#[test]
fn test_int_power() {
    let out = jit_capture(
        r#"print(2 ** 10)
print(3 ** 4)
print(5 ** 0)
"#,
    );
    assert_output(&out, "1024\n81\n1\n");
}

/// Ported from `Lib/test/test_int_ported.py`.
#[test]
fn test_int_unary_negation() {
    let out = jit_capture(
        r#"x = 5
print(-x)
print(-(-x))
"#,
    );
    assert_output(&out, "-5\n5\n");
}

/// Ported from `Lib/test/test_int_ported.py`.
#[test]
fn test_int_less_than() {
    let out = jit_capture(
        r#"print(1 < 2)
print(2 < 1)
print(2 < 2)
"#,
    );
    assert_output(&out, "True\nFalse\nFalse\n");
}

/// Ported from `Lib/test/test_int_ported.py`.
#[test]
fn test_int_greater_than() {
    let out = jit_capture(
        r#"print(2 > 1)
print(1 > 2)
print(2 > 2)
"#,
    );
    assert_output(&out, "True\nFalse\nFalse\n");
}

/// Ported from `Lib/test/test_int_ported.py`.
#[test]
fn test_int_less_equal() {
    let out = jit_capture(
        r#"print(2 <= 2)
print(2 <= 3)
print(3 <= 2)
"#,
    );
    assert_output(&out, "True\nTrue\nFalse\n");
}

/// Ported from `Lib/test/test_int_ported.py`.
#[test]
fn test_int_greater_equal() {
    let out = jit_capture(
        r#"print(2 >= 2)
print(3 >= 2)
print(2 >= 3)
"#,
    );
    assert_output(&out, "True\nTrue\nFalse\n");
}

/// Ported from `Lib/test/test_int_ported.py`.
#[test]
fn test_int_equality() {
    let out = jit_capture(
        r#"print(42 == 42)
print(42 == 43)
print(-1 == -1)
"#,
    );
    assert_output(&out, "True\nFalse\nTrue\n");
}

/// Ported from `Lib/test/test_int_ported.py`.
#[test]
fn test_int_from_string() {
    let out = jit_capture(
        r#"print(int("42"))
print(int("-7"))
print(int("0"))
"#,
    );
    assert_output(&out, "42\n-7\n0\n");
}

/// Ported from `Lib/test/test_int_ported.py`.
#[test]
fn test_int_default_is_zero() {
    let out = jit_capture(
        r#"print(int())
"#,
    );
    assert_output(&out, "0\n");
}

/// Ported from `Lib/test/test_int_ported.py`.
#[test]
fn test_int_abs_positive() {
    let out = jit_capture(
        r#"print(abs(42))
"#,
    );
    assert_output(&out, "42\n");
}

/// Ported from `Lib/test/test_int_ported.py`.
#[test]
fn test_int_abs_negative() {
    let out = jit_capture(
        r#"print(abs(-42))
"#,
    );
    assert_output(&out, "42\n");
}

/// Ported from `Lib/test/test_int_ported.py`.
#[test]
fn test_int_abs_zero() {
    let out = jit_capture(
        r#"print(abs(0))
"#,
    );
    assert_output(&out, "0\n");
}

/// Ported from `Lib/test/test_int_ported.py`.
#[test]
fn test_int_bool_nonzero_is_true() {
    let out = jit_capture(
        r#"print(bool(1))
print(bool(-5))
"#,
    );
    assert_output(&out, "True\nTrue\n");
}

/// Ported from `Lib/test/test_int_ported.py`.
#[test]
fn test_int_bitwise_and() {
    let out = jit_capture(
        r#"print(0b1100 & 0b1010)
"#,
    );
    assert_output(&out, "8\n");
}

/// Ported from `Lib/test/test_int_ported.py`.
#[test]
fn test_int_bitwise_or() {
    let out = jit_capture(
        r#"print(0b1100 | 0b1010)
"#,
    );
    assert_output(&out, "14\n");
}

/// Ported from `Lib/test/test_int_ported.py`.
#[test]
fn test_int_bitwise_xor() {
    let out = jit_capture(
        r#"print(0b1100 ^ 0b1010)
"#,
    );
    assert_output(&out, "6\n");
}

/// Ported from `Lib/test/test_int_ported.py`.
#[test]
fn test_int_left_shift() {
    let out = jit_capture(
        r#"print(1 << 4)
print(3 << 2)
"#,
    );
    assert_output(&out, "16\n12\n");
}

/// Ported from `Lib/test/test_int_ported.py`.
#[test]
fn test_int_right_shift() {
    let out = jit_capture(
        r#"print(16 >> 2)
print(100 >> 1)
"#,
    );
    assert_output(&out, "4\n50\n");
}

/// Ported from `Lib/test/test_int_ported.py`.
#[test]
fn test_int_divmod_positive() {
    let out = jit_capture(
        r#"q, r = divmod(17, 5)
print(q)
print(r)
"#,
    );
    assert_output(&out, "3\n2\n");
}

/// Ported from `Lib/test/test_int_ported.py`.
#[test]
fn test_int_precedence_mul_before_add() {
    let out = jit_capture(
        r#"print(2 + 3 * 4)
"#,
    );
    assert_output(&out, "14\n");
}

/// Ported from `Lib/test/test_int_ported.py`.
#[test]
fn test_int_precedence_parens_override() {
    let out = jit_capture(
        r#"print((2 + 3) * 4)
"#,
    );
    assert_output(&out, "20\n");
}

/// Ported from `Lib/test/test_int_ported.py`.
#[test]
fn test_int_precedence_pow_right_assoc() {
    let out = jit_capture(
        r#"print(2 ** 3 ** 2)
"#,
    );
    assert_output(&out, "512\n");
}

/// Ported from `Lib/test/test_int_ported.py`.
#[test]
fn test_int_augmented_add() {
    let out = jit_capture(
        r#"x = 10
x += 5
print(x)
"#,
    );
    assert_output(&out, "15\n");
}

/// Ported from `Lib/test/test_int_ported.py`.
#[test]
fn test_int_augmented_sub() {
    let out = jit_capture(
        r#"x = 10
x -= 3
print(x)
"#,
    );
    assert_output(&out, "7\n");
}

/// Ported from `Lib/test/test_int_ported.py`.
#[test]
fn test_int_augmented_mul() {
    let out = jit_capture(
        r#"x = 5
x *= 4
print(x)
"#,
    );
    assert_output(&out, "20\n");
}

/// Ported from `Lib/test/test_int_ported.py`.
#[test]
fn test_int_floor_division_2() {
    let out = jit_capture(
        r#"print(10 // 3)
print(-10 // 3)
print(10 // -3)
print(7 // 2)
"#,
    );
    assert_output(&out, "3\n-4\n-4\n3\n");
}

/// Ported from `Lib/test/test_int_ported.py`.
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

/// Ported from `Lib/test/test_int_ported.py`.
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

/// Ported from `Lib/test/test_int_ported.py`.
#[test]
fn test_int_unary_negation_2() {
    let out = jit_capture(
        r#"print(-(-5))
print(-0)
print(-(7))
print(-(-(-3)))
"#,
    );
    assert_output(&out, "5\n0\n-7\n-3\n");
}


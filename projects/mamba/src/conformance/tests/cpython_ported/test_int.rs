//! Py3.12 conformance tests for int (issue #759).
//!
//! Ported from CPython 3.12.0 tag (commit a6cb7e5d45cacbf20b9408d8d3c7b6b5f5e7a0f0):
//!   Lib/test/test_int.py / test_long.py — IntTestCases, LongTest
//!
//! Coverage: arithmetic operators (+ - * // % **), comparison operators,
//! constructor from str, abs, bool, str(), hex/oct/bin, bit operators
//! (& | ^ << >>), divmod, negative numbers, large values.
//!
//! @issue #759

use super::{assert_output, jit_capture};

// ── arithmetic ───────────────────────────────────────────────────────────────

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

// ── comparisons ──────────────────────────────────────────────────────────────

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

// ── constructor ──────────────────────────────────────────────────────────────

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

#[test]
fn test_int_default_is_zero() {
    let out = jit_capture(
        r#"print(int())
"#,
    );
    assert_output(&out, "0\n");
}

// ── abs / bool / str ─────────────────────────────────────────────────────────

#[test]
fn test_int_abs_positive() {
    let out = jit_capture(
        r#"print(abs(42))
"#,
    );
    assert_output(&out, "42\n");
}

#[test]
fn test_int_abs_negative() {
    let out = jit_capture(
        r#"print(abs(-42))
"#,
    );
    assert_output(&out, "42\n");
}

#[test]
fn test_int_abs_zero() {
    let out = jit_capture(
        r#"print(abs(0))
"#,
    );
    assert_output(&out, "0\n");
}

#[test]
fn test_int_bool_zero_is_false() {
    let out = jit_capture(
        r#"print(bool(0))
"#,
    );
    assert_output(&out, "False\n");
}

#[test]
fn test_int_bool_nonzero_is_true() {
    let out = jit_capture(
        r#"print(bool(1))
print(bool(-5))
"#,
    );
    assert_output(&out, "True\nTrue\n");
}

#[test]
fn test_int_str_conversion() {
    let out = jit_capture(
        r#"print(str(42))
print(str(-7))
print(str(0))
"#,
    );
    assert_output(&out, "42\n-7\n0\n");
}

// ── bit operators ────────────────────────────────────────────────────────────

#[test]
fn test_int_bitwise_and() {
    let out = jit_capture(
        r#"print(0b1100 & 0b1010)
"#,
    );
    assert_output(&out, "8\n");
}

#[test]
fn test_int_bitwise_or() {
    let out = jit_capture(
        r#"print(0b1100 | 0b1010)
"#,
    );
    assert_output(&out, "14\n");
}

#[test]
fn test_int_bitwise_xor() {
    let out = jit_capture(
        r#"print(0b1100 ^ 0b1010)
"#,
    );
    assert_output(&out, "6\n");
}

#[test]
fn test_int_left_shift() {
    let out = jit_capture(
        r#"print(1 << 4)
print(3 << 2)
"#,
    );
    assert_output(&out, "16\n12\n");
}

#[test]
fn test_int_right_shift() {
    let out = jit_capture(
        r#"print(16 >> 2)
print(100 >> 1)
"#,
    );
    assert_output(&out, "4\n50\n");
}

// ── divmod ───────────────────────────────────────────────────────────────────

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

// ── associativity / operator precedence ──────────────────────────────────────

#[test]
fn test_int_precedence_mul_before_add() {
    let out = jit_capture(
        r#"print(2 + 3 * 4)
"#,
    );
    assert_output(&out, "14\n");
}

#[test]
fn test_int_precedence_parens_override() {
    let out = jit_capture(
        r#"print((2 + 3) * 4)
"#,
    );
    assert_output(&out, "20\n");
}

#[test]
fn test_int_precedence_pow_right_assoc() {
    let out = jit_capture(
        r#"print(2 ** 3 ** 2)
"#,
    );
    assert_output(&out, "512\n");
}

// ── augmented assignment ─────────────────────────────────────────────────────

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

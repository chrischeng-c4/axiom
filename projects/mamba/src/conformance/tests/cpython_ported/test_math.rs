//! Py3.12 conformance tests for the `math` module (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_math.py):
//!   MathTests basic surface — constants, sqrt/floor/ceil/trunc,
//!   exponent/log family, trig, gcd/lcm/isqrt, factorial/comb/perm,
//!   degrees/radians/hypot/atan2, isnan/isinf/isfinite, fabs/copysign/fmod,
//!   pow, prod.
//!
//! Companion to `test_float.rs` and `test_float_extended.rs`.
//!
//! @issue #759

use super::{assert_output, jit_capture};

// ---------------------------------------------------------------- constants

#[test]
fn test_math_pi() {
    let out = jit_capture("import math\nprint(math.pi)\n");
    assert_output(&out, "3.141592653589793\n");
}

#[test]
fn test_math_e() {
    let out = jit_capture("import math\nprint(math.e)\n");
    assert_output(&out, "2.718281828459045\n");
}

#[test]
fn test_math_tau() {
    let out = jit_capture("import math\nprint(math.tau)\n");
    assert_output(&out, "6.283185307179586\n");
}

#[test]
fn test_math_inf_constant() {
    let out = jit_capture("import math\nprint(math.inf)\n");
    assert_output(&out, "inf\n");
}

#[test]
fn test_math_nan_constant() {
    let out = jit_capture("import math\nprint(math.nan)\n");
    assert_output(&out, "nan\n");
}

// ---------------------------------------------------------------- sqrt/round family

#[test]
fn test_math_sqrt() {
    let out = jit_capture("import math\nprint(math.sqrt(16.0))\n");
    assert_output(&out, "4.0\n");
}

#[test]
fn test_math_sqrt_int_arg() {
    let out = jit_capture("import math\nprint(math.sqrt(4))\n");
    assert_output(&out, "2.0\n");
}

#[test]
fn test_math_floor() {
    let out = jit_capture("import math\nprint(math.floor(3.7))\n");
    assert_output(&out, "3\n");
}

#[test]
fn test_math_ceil() {
    let out = jit_capture("import math\nprint(math.ceil(3.2))\n");
    assert_output(&out, "4\n");
}

#[test]
fn test_math_trunc_positive() {
    let out = jit_capture("import math\nprint(math.trunc(3.7))\n");
    assert_output(&out, "3\n");
}

#[test]
fn test_math_trunc_negative() {
    let out = jit_capture("import math\nprint(math.trunc(-3.7))\n");
    assert_output(&out, "-3\n");
}

// ---------------------------------------------------------------- fabs/copysign/fmod

#[test]
fn test_math_fabs() {
    let out = jit_capture("import math\nprint(math.fabs(-5.5))\n");
    assert_output(&out, "5.5\n");
}

#[test]
fn test_math_copysign_negative() {
    let out = jit_capture("import math\nprint(math.copysign(3.0, -1.0))\n");
    assert_output(&out, "-3.0\n");
}

#[test]
fn test_math_fmod() {
    let out = jit_capture("import math\nprint(math.fmod(10, 3))\n");
    assert_output(&out, "1.0\n");
}

// ---------------------------------------------------------------- pow

#[test]
fn test_math_pow_floats() {
    let out = jit_capture("import math\nprint(math.pow(2.0, 10.0))\n");
    assert_output(&out, "1024.0\n");
}

#[test]
fn test_math_pow_ints_returns_float() {
    // math.pow always returns float, unlike `**`.
    let out = jit_capture("import math\nprint(math.pow(2, 3))\n");
    assert_output(&out, "8.0\n");
}

// ---------------------------------------------------------------- exp/log family

#[test]
fn test_math_exp_zero() {
    let out = jit_capture("import math\nprint(math.exp(0))\n");
    assert_output(&out, "1.0\n");
}

#[test]
fn test_math_log_e() {
    let out = jit_capture("import math\nprint(math.log(math.e))\n");
    assert_output(&out, "1.0\n");
}

#[test]
fn test_math_log_one() {
    let out = jit_capture("import math\nprint(math.log(1))\n");
    assert_output(&out, "0.0\n");
}

#[test]
fn test_math_log2() {
    let out = jit_capture("import math\nprint(math.log2(8))\n");
    assert_output(&out, "3.0\n");
}

#[test]
fn test_math_log10() {
    let out = jit_capture("import math\nprint(math.log10(1000))\n");
    assert_output(&out, "3.0\n");
}

// ---------------------------------------------------------------- trig + angle conversion

#[test]
fn test_math_sin_zero() {
    let out = jit_capture("import math\nprint(round(math.sin(0), 10))\n");
    assert_output(&out, "0.0\n");
}

#[test]
fn test_math_cos_zero() {
    let out = jit_capture("import math\nprint(round(math.cos(0), 10))\n");
    assert_output(&out, "1.0\n");
}

#[test]
fn test_math_degrees_pi() {
    let out = jit_capture("import math\nprint(math.degrees(math.pi))\n");
    assert_output(&out, "180.0\n");
}

#[test]
fn test_math_radians_180() {
    let out = jit_capture("import math\nprint(math.radians(180.0))\n");
    assert_output(&out, "3.141592653589793\n");
}

#[test]
fn test_math_atan2_45() {
    let out = jit_capture("import math\nprint(math.atan2(1.0, 1.0))\n");
    assert_output(&out, "0.7853981633974483\n");
}

#[test]
fn test_math_hypot_3_4() {
    let out = jit_capture("import math\nprint(math.hypot(3.0, 4.0))\n");
    assert_output(&out, "5.0\n");
}

// ---------------------------------------------------------------- gcd/lcm/isqrt

#[test]
fn test_math_gcd_pair() {
    let out = jit_capture("import math\nprint(math.gcd(12, 18))\n");
    assert_output(&out, "6\n");
}

#[test]
fn test_math_gcd_variadic() {
    let out = jit_capture("import math\nprint(math.gcd(12, 18, 24))\n");
    assert_output(&out, "6\n");
}

#[test]
fn test_math_lcm() {
    let out = jit_capture("import math\nprint(math.lcm(4, 6))\n");
    assert_output(&out, "12\n");
}

#[test]
fn test_math_isqrt() {
    let out = jit_capture("import math\nprint(math.isqrt(17))\n");
    assert_output(&out, "4\n");
}

// ---------------------------------------------------------------- factorial/comb/perm

#[test]
fn test_math_factorial_5() {
    let out = jit_capture("import math\nprint(math.factorial(5))\n");
    assert_output(&out, "120\n");
}

#[test]
fn test_math_factorial_10() {
    let out = jit_capture("import math\nprint(math.factorial(10))\n");
    assert_output(&out, "3628800\n");
}

#[test]
fn test_math_comb() {
    let out = jit_capture("import math\nprint(math.comb(5, 2))\n");
    assert_output(&out, "10\n");
}

#[test]
fn test_math_perm() {
    let out = jit_capture("import math\nprint(math.perm(5, 2))\n");
    assert_output(&out, "20\n");
}

// ---------------------------------------------------------------- predicates

#[test]
fn test_math_isnan_true() {
    let out = jit_capture(
        r#"import math
print(math.isnan(float("nan")))
"#,
    );
    assert_output(&out, "True\n");
}

#[test]
fn test_math_isnan_false() {
    let out = jit_capture("import math\nprint(math.isnan(1.0))\n");
    assert_output(&out, "False\n");
}

#[test]
fn test_math_isinf_true() {
    let out = jit_capture(
        r#"import math
print(math.isinf(float("inf")))
"#,
    );
    assert_output(&out, "True\n");
}

#[test]
fn test_math_isinf_false() {
    let out = jit_capture("import math\nprint(math.isinf(1.0))\n");
    assert_output(&out, "False\n");
}

#[test]
fn test_math_isfinite_true() {
    let out = jit_capture("import math\nprint(math.isfinite(1.0))\n");
    assert_output(&out, "True\n");
}

// ---------------------------------------------------------------- prod

#[test]
fn test_math_prod() {
    let out = jit_capture("import math\nprint(math.prod([1, 2, 3, 4]))\n");
    assert_output(&out, "24\n");
}

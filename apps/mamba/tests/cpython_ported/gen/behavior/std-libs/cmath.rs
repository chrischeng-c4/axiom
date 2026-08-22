use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/cmath/abs_infinite_component_is_inf.py`.
#[test]
fn test_gen_behavior_std_libs_cmath_abs_infinite_component_is_inf() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "behavior"
# case = "abs_infinite_component_is_inf"
# subject = "abs"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""abs: abs() of any complex with an infinite component is +inf, and an infinite component beats a NaN component"""
import math

INF = float("inf")
NAN = float("nan")

assert abs(complex(INF, 2.3)) == INF, "abs with real inf is inf"
assert abs(complex(2.3, -INF)) == INF, "abs with imag -inf is inf"
assert abs(complex(-INF, -INF)) == INF, "abs of inf+inf is inf"
assert abs(complex(NAN, -INF)) == INF, "inf beats nan in abs"
assert abs(complex(INF, NAN)) == INF, "inf beats nan in abs (real side)"
print("abs_infinite_component_is_inf OK")
"###);
    assert_output(&out, r###"abs_infinite_component_is_inf OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/cmath/abs_nan_without_inf_is_nan.py`.
#[test]
fn test_gen_behavior_std_libs_cmath_abs_nan_without_inf_is_nan() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "behavior"
# case = "abs_nan_without_inf_is_nan"
# subject = "abs"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""abs: abs() is NaN when a component is NaN and no component is infinite"""
import math

NAN = float("nan")

assert math.isnan(abs(complex(NAN, 2.3))), "abs(nan+2.3j) is nan"
assert math.isnan(abs(complex(-2.3, NAN))), "abs(-2.3+nanj) is nan"
assert math.isnan(abs(complex(NAN, NAN))), "abs(nan+nanj) is nan"
print("abs_nan_without_inf_is_nan OK")
"###);
    assert_output(&out, r###"abs_nan_without_inf_is_nan OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/cmath/abs_zero_is_zero.py`.
#[test]
fn test_gen_behavior_std_libs_cmath_abs_zero_is_zero() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "behavior"
# case = "abs_zero_is_zero"
# subject = "abs"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""abs: abs() of complex zero (and signed zero) is 0.0"""
import cmath  # noqa: F401

assert abs(complex(0.0, 0.0)) == 0.0, "abs of zero is zero"
assert abs(complex(-0.0, -0.0)) == 0.0, "abs of signed zero is zero"
print("abs_zero_is_zero OK")
"###);
    assert_output(&out, r###"abs_zero_is_zero OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/cmath/complex_multiply_algebraic.py`.
#[test]
fn test_gen_behavior_std_libs_cmath_complex_multiply_algebraic() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "behavior"
# case = "complex_multiply_algebraic"
# subject = "cmath"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cmath: complex multiplication follows the algebraic rule: (1+2j)*(3+4j) == -5+10j"""
import cmath  # noqa: F401

_a = 1 + 2j
_b = 3 + 4j
# (1+2j)(3+4j) = 3+4j+6j+8j^2 = 3+10j-8 = -5+10j
assert _a * _b == -5 + 10j, f"complex multiply = {_a * _b!r}"
print("complex_multiply_algebraic OK")
"###);
    assert_output(&out, r###"complex_multiply_algebraic OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/cmath/exp_euler_identity.py`.
#[test]
fn test_gen_behavior_std_libs_cmath_exp_euler_identity() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "behavior"
# case = "exp_euler_identity"
# subject = "cmath.exp"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cmath.exp: Euler's identity: exp(j*pi) + 1 has magnitude ~ 0"""
import cmath
import math

_euler = cmath.exp(1j * math.pi) + 1
assert abs(_euler) < 1e-12, f"|exp(j*pi)+1| = {abs(_euler)!r}"
print("exp_euler_identity OK")
"###);
    assert_output(&out, r###"exp_euler_identity OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/cmath/inf_lives_on_real_axis.py`.
#[test]
fn test_gen_behavior_std_libs_cmath_inf_lives_on_real_axis() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "behavior"
# case = "inf_lives_on_real_axis"
# subject = "cmath.inf"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cmath.inf: cmath.inf has +inf real part and 0 imaginary part; infj is the imaginary-axis mirror"""
import cmath
import math

assert cmath.inf.real == math.inf, f"inf.real = {cmath.inf.real!r}"
assert cmath.inf.imag == 0.0, f"inf.imag = {cmath.inf.imag!r}"
assert cmath.infj.real == 0.0, f"infj.real = {cmath.infj.real!r}"
assert cmath.infj.imag == math.inf, f"infj.imag = {cmath.infj.imag!r}"
print("inf_lives_on_real_axis OK")
"###);
    assert_output(&out, r###"inf_lives_on_real_axis OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/cmath/isclose_abs_tol_near_and_far.py`.
#[test]
fn test_gen_behavior_std_libs_cmath_isclose_abs_tol_near_and_far() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "behavior"
# case = "isclose_abs_tol_near_and_far"
# subject = "cmath.isclose"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cmath.isclose: isclose with abs_tol: nearby values are close, far-apart values are not"""
import cmath

assert cmath.isclose(1j, 1.0001j, abs_tol=0.001), "isclose abs_tol near"
assert not cmath.isclose(1j, 2j, abs_tol=0.001), "not isclose far apart"
print("isclose_abs_tol_near_and_far OK")
"###);
    assert_output(&out, r###"isclose_abs_tol_near_and_far OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/cmath/isclose_rel_tol_near.py`.
#[test]
fn test_gen_behavior_std_libs_cmath_isclose_rel_tol_near() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "behavior"
# case = "isclose_rel_tol_near"
# subject = "cmath.isclose"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cmath.isclose: isclose with rel_tol accepts two nearly-equal complex values"""
import cmath

assert cmath.isclose(1 + 1j, 1 + 1.0000001j, rel_tol=1e-5), "isclose rel_tol near"
print("isclose_rel_tol_near OK")
"###);
    assert_output(&out, r###"isclose_rel_tol_near OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/cmath/isfinite_only_for_finite.py`.
#[test]
fn test_gen_behavior_std_libs_cmath_isfinite_only_for_finite() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "behavior"
# case = "isfinite_only_for_finite"
# subject = "cmath.isfinite"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cmath.isfinite: isfinite is True for a finite complex and False once a component is infinite"""
import cmath

assert cmath.isfinite(complex(1, 2)), "isfinite(1+2j)"
assert not cmath.isfinite(complex(float("inf"), 0)), "not isfinite inf"
print("isfinite_only_for_finite OK")
"###);
    assert_output(&out, r###"isfinite_only_for_finite OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/cmath/isinf_detects_infinite_component.py`.
#[test]
fn test_gen_behavior_std_libs_cmath_isinf_detects_infinite_component() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "behavior"
# case = "isinf_detects_infinite_component"
# subject = "cmath.isinf"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cmath.isinf: isinf is True when either the real or imaginary component is infinite, and accepts plain int/float too"""
import cmath

assert cmath.isinf(complex(float("inf"), 0)), "isinf real inf"
assert cmath.isinf(complex(0, float("inf"))), "isinf imag inf"
assert cmath.isinf(float("inf")), "isinf(float inf)"
# Finite ints and the finite imaginary unit are not infinite.
assert not cmath.isinf(1), "isinf(int 1) is False"
assert not cmath.isinf(1j), "isinf(1j) is False"
print("isinf_detects_infinite_component OK")
"###);
    assert_output(&out, r###"isinf_detects_infinite_component OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/cmath/isnan_detects_nan_component.py`.
#[test]
fn test_gen_behavior_std_libs_cmath_isnan_detects_nan_component() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "behavior"
# case = "isnan_detects_nan_component"
# subject = "cmath.isnan"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cmath.isnan: isnan is True when a component is NaN and False for finite int/complex; accepts float('nan')"""
import cmath

assert cmath.isnan(complex(float("nan"), 0)), "isnan real nan"
assert cmath.isnan(float("nan")), "isnan(float nan)"
assert not cmath.isnan(1), "isnan(int 1) is False"
print("isnan_detects_nan_component OK")
"###);
    assert_output(&out, r###"isnan_detects_nan_component OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/cmath/log_inverts_exp.py`.
#[test]
fn test_gen_behavior_std_libs_cmath_log_inverts_exp() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "behavior"
# case = "log_inverts_exp"
# subject = "cmath.log"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cmath.log: log is the inverse of exp: log(exp(1+1j)) == 1+1j to tolerance"""
import cmath

_z = 1 + 1j
assert abs(cmath.log(cmath.exp(_z)) - _z) < 1e-12, "log(exp(z)) = z"
print("log_inverts_exp OK")
"###);
    assert_output(&out, r###"log_inverts_exp OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/cmath/log_neg_one_is_pi_j.py`.
#[test]
fn test_gen_behavior_std_libs_cmath_log_neg_one_is_pi_j() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "behavior"
# case = "log_neg_one_is_pi_j"
# subject = "cmath.log"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cmath.log: log(-1) == pi*j on the principal branch"""
import cmath
import math

assert abs(cmath.log(-1) - 1j * math.pi) < 1e-15, f"log(-1) = {cmath.log(-1)!r}"
print("log_neg_one_is_pi_j OK")
"###);
    assert_output(&out, r###"log_neg_one_is_pi_j OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/cmath/log_one_is_zero.py`.
#[test]
fn test_gen_behavior_std_libs_cmath_log_one_is_zero() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "behavior"
# case = "log_one_is_zero"
# subject = "cmath.log"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cmath.log: log(1) == 0 (zero magnitude)"""
import cmath

assert abs(cmath.log(1)) < 1e-15, f"log(1) = {cmath.log(1)!r}"
print("log_one_is_zero OK")
"###);
    assert_output(&out, r###"log_one_is_zero OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/cmath/nan_carries_one_nan_component.py`.
#[test]
fn test_gen_behavior_std_libs_cmath_nan_carries_one_nan_component() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "behavior"
# case = "nan_carries_one_nan_component"
# subject = "cmath.nan"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cmath.nan: cmath.nan carries NaN in the real component and a clean +0 imaginary; nanj is the imaginary-axis mirror"""
import cmath
import math

assert math.isnan(cmath.nan.real), "nan.real is NaN"
assert cmath.nan.imag == 0.0, f"nan.imag = {cmath.nan.imag!r}"
assert cmath.nanj.real == 0.0, f"nanj.real = {cmath.nanj.real!r}"
assert math.isnan(cmath.nanj.imag), "nanj.imag is NaN"
print("nan_carries_one_nan_component OK")
"###);
    assert_output(&out, r###"nan_carries_one_nan_component OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/cmath/nan_constants_positive_sign_bits.py`.
#[test]
fn test_gen_behavior_std_libs_cmath_nan_constants_positive_sign_bits() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "behavior"
# case = "nan_constants_positive_sign_bits"
# subject = "cmath.nan"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cmath.nan: every component of the cmath.nan / cmath.nanj constants has a positive sign bit (copysign(1, c) == 1)"""
import cmath
import math

assert math.copysign(1.0, cmath.nan.real) == 1.0, "nan.real sign +"
assert math.copysign(1.0, cmath.nan.imag) == 1.0, "nan.imag sign +"
assert math.copysign(1.0, cmath.nanj.real) == 1.0, "nanj.real sign +"
assert math.copysign(1.0, cmath.nanj.imag) == 1.0, "nanj.imag sign +"
print("nan_constants_positive_sign_bits OK")
"###);
    assert_output(&out, r###"nan_constants_positive_sign_bits OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/cmath/phase_at_infinity_compass_angles.py`.
#[test]
fn test_gen_behavior_std_libs_cmath_phase_at_infinity_compass_angles() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "behavior"
# case = "phase_at_infinity_compass_angles"
# subject = "cmath.phase"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cmath.phase: phase() of directions at infinity collapses to the compass angles (pi/4, 3pi/4, etc.); a finite component is dwarfed by an infinite one"""
import cmath
import math

INF = float("inf")

assert abs(cmath.phase(complex(INF, INF)) - math.pi / 4) < 1e-9, "phase NE = pi/4"
assert abs(cmath.phase(complex(-INF, INF)) - 0.75 * math.pi) < 1e-9, "phase NW"
assert abs(cmath.phase(complex(-INF, -INF)) + 0.75 * math.pi) < 1e-9, "phase SW"
assert abs(cmath.phase(complex(INF, -INF)) + math.pi / 4) < 1e-9, "phase SE"
# A finite component is dwarfed by an infinite one.
assert abs(cmath.phase(complex(2.3, INF)) - math.pi / 2) < 1e-9, "phase up = pi/2"
assert cmath.phase(complex(INF, 2.3)) == 0.0, "phase right = 0"
assert abs(cmath.phase(complex(-INF, 2.3)) - math.pi) < 1e-9, "phase left = pi"
print("phase_at_infinity_compass_angles OK")
"###);
    assert_output(&out, r###"phase_at_infinity_compass_angles OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/cmath/phase_nan_component_is_nan.py`.
#[test]
fn test_gen_behavior_std_libs_cmath_phase_nan_component_is_nan() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "behavior"
# case = "phase_nan_component_is_nan"
# subject = "cmath.phase"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cmath.phase: phase() is NaN when any component is NaN"""
import cmath
import math

NAN = float("nan")

assert math.isnan(cmath.phase(complex(NAN, 1.0))), "phase(nan+1j) is nan"
assert math.isnan(cmath.phase(complex(1.0, NAN))), "phase(1+nanj) is nan"
assert math.isnan(cmath.phase(complex(NAN, NAN))), "phase(nan+nanj) is nan"
print("phase_nan_component_is_nan OK")
"###);
    assert_output(&out, r###"phase_nan_component_is_nan OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/cmath/phase_round_trip_in_principal_range.py`.
#[test]
fn test_gen_behavior_std_libs_cmath_phase_round_trip_in_principal_range() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "behavior"
# case = "phase_round_trip_in_principal_range"
# subject = "cmath.phase"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cmath.phase: phase of rect(1, theta) recovers theta (mapped into the principal range (-pi, pi]) across a table of angles"""
import cmath
import math

for _theta in [-math.pi, -math.pi / 2, 0, math.pi / 2, math.pi]:
    _z = cmath.rect(1, _theta)
    _p = cmath.phase(_z)
    # phase of rect(1, theta) = theta (modulo 2pi, mapped to (-pi, pi]).
    assert abs(_p - _theta) < 1e-12 or abs(abs(_p) - math.pi) < 1e-12, \
        f"phase round-trip {_theta}"
print("phase_round_trip_in_principal_range OK")
"###);
    assert_output(&out, r###"phase_round_trip_in_principal_range OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/cmath/pi_e_match_math.py`.
#[test]
fn test_gen_behavior_std_libs_cmath_pi_e_match_math() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "behavior"
# case = "pi_e_match_math"
# subject = "cmath.pi"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cmath.pi: cmath.pi and cmath.e agree with the math module values to float tolerance"""
import cmath
import math

assert abs(cmath.pi - math.pi) < 1e-15, f"cmath.pi = {cmath.pi!r}"
assert abs(cmath.e - math.e) < 1e-15, f"cmath.e = {cmath.e!r}"
print("pi_e_match_math OK")
"###);
    assert_output(&out, r###"pi_e_match_math OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/cmath/polar_rect_round_trip.py`.
#[test]
fn test_gen_behavior_std_libs_cmath_polar_rect_round_trip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "behavior"
# case = "polar_rect_round_trip"
# subject = "cmath.polar"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cmath.polar: polar and rect are inverse operations across a table of representative complex points"""
import cmath

for _x, _y in [(1, 0), (0, 1), (-1, 0), (1, 1), (3, 4)]:
    _z = complex(_x, _y)
    _r, _phi = cmath.polar(_z)
    _z2 = cmath.rect(_r, _phi)
    assert abs(_z - _z2) < 1e-12, f"polar/rect round-trip {_z}"
print("polar_rect_round_trip OK")
"###);
    assert_output(&out, r###"polar_rect_round_trip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/cmath/sin_cos_pythagorean_identity.py`.
#[test]
fn test_gen_behavior_std_libs_cmath_sin_cos_pythagorean_identity() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "behavior"
# case = "sin_cos_pythagorean_identity"
# subject = "cmath.sin"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cmath.sin: the Pythagorean identity sin(z)**2 + cos(z)**2 == 1 holds for a complex argument"""
import cmath

_z = 1 + 2j
_sq_sum = cmath.sin(_z) ** 2 + cmath.cos(_z) ** 2
assert abs(_sq_sum - 1) < 1e-12, f"sin^2+cos^2 = 1 for complex z: {_sq_sum!r}"
print("sin_cos_pythagorean_identity OK")
"###);
    assert_output(&out, r###"sin_cos_pythagorean_identity OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/cmath/special_constant_reprs.py`.
#[test]
fn test_gen_behavior_std_libs_cmath_special_constant_reprs() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "behavior"
# case = "special_constant_reprs"
# subject = "cmath.inf"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cmath.inf: repr of the special constants uses the compact spellings 'inf', 'infj', 'nan', 'nanj'"""
import cmath

assert repr(cmath.inf) == "inf", f"repr inf = {repr(cmath.inf)!r}"
assert repr(cmath.infj) == "infj", f"repr infj = {repr(cmath.infj)!r}"
assert repr(cmath.nan) == "nan", f"repr nan = {repr(cmath.nan)!r}"
assert repr(cmath.nanj) == "nanj", f"repr nanj = {repr(cmath.nanj)!r}"
print("special_constant_reprs OK")
"###);
    assert_output(&out, r###"special_constant_reprs OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/cmath/sqrt_neg_four_is_two_j.py`.
#[test]
fn test_gen_behavior_std_libs_cmath_sqrt_neg_four_is_two_j() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "behavior"
# case = "sqrt_neg_four_is_two_j"
# subject = "cmath.sqrt"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cmath.sqrt: sqrt(-4) == 2j on the principal branch"""
import cmath

assert abs(cmath.sqrt(-4) - 2j) < 1e-14, f"sqrt(-4) = {cmath.sqrt(-4)!r}"
print("sqrt_neg_four_is_two_j OK")
"###);
    assert_output(&out, r###"sqrt_neg_four_is_two_j OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/cmath/sqrt_neg_one_is_imaginary_unit.py`.
#[test]
fn test_gen_behavior_std_libs_cmath_sqrt_neg_one_is_imaginary_unit() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "behavior"
# case = "sqrt_neg_one_is_imaginary_unit"
# subject = "cmath.sqrt"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cmath.sqrt: sqrt(-1) is the imaginary unit 1j: real part ~ 0, imag part ~ 1"""
import cmath

_sq = cmath.sqrt(-1)
assert abs(_sq.real) < 1e-15, f"sqrt(-1).real ~ 0 = {_sq.real!r}"
assert abs(_sq.imag - 1) < 1e-15, "sqrt(-1).imag = 1"
print("sqrt_neg_one_is_imaginary_unit OK")
"###);
    assert_output(&out, r###"sqrt_neg_one_is_imaginary_unit OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/cmath/sqrt_positive_real_result.py`.
#[test]
fn test_gen_behavior_std_libs_cmath_sqrt_positive_real_result() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "behavior"
# case = "sqrt_positive_real_result"
# subject = "cmath.sqrt"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cmath.sqrt: sqrt(4) returns a complex equal to 2 (zero imaginary part)"""
import cmath

_sq = cmath.sqrt(4)
assert isinstance(_sq, complex), f"sqrt(4) type = {type(_sq)!r}"
assert abs(_sq - 2) < 1e-15, f"sqrt(4) = {_sq!r}"
print("sqrt_positive_real_result OK")
"###);
    assert_output(&out, r###"sqrt_positive_real_result OK
"###);
}

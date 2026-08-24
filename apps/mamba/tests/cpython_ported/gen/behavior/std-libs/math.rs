use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/math/comb_perm.py`.
#[test]
fn test_gen_behavior_std_libs_math_comb_perm() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "behavior"
# case = "comb_perm"
# subject = "math.comb"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.comb: binomial coefficient and permutations: comb(5, 2)==10, comb(10, 0)==1, perm(5, 2)==20"""
import math

assert math.comb(5, 2) == 10, f"comb(5,2) = {math.comb(5, 2)!r}"
assert math.comb(10, 0) == 1, f"comb(10,0) = {math.comb(10, 0)!r}"
assert math.perm(5, 2) == 20, f"perm(5,2) = {math.perm(5, 2)!r}"

print("comb_perm OK")
"###);
    assert_output(&out, r###"comb_perm OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/math/constants_values.py`.
#[test]
fn test_gen_behavior_std_libs_math_constants_values() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "behavior"
# case = "constants_values"
# subject = "math.pi"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.pi: the named constants carry their canonical IEEE-754 values: pi==3.141592653589793, e==2.718281828459045, tau==2*pi; inf is infinite and nan != nan"""
import math

assert math.pi == 3.141592653589793, f"pi = {math.pi!r}"
assert math.e == 2.718281828459045, f"e = {math.e!r}"
assert abs(math.tau - 2 * math.pi) < 1e-10, f"tau = {math.tau!r}"
assert math.isinf(math.inf), "inf is infinite"
assert math.nan != math.nan, "nan != nan"

print("constants_values OK")
"###);
    assert_output(&out, r###"constants_values OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/math/copysign_carries_sign.py`.
#[test]
fn test_gen_behavior_std_libs_math_copysign_carries_sign() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "behavior"
# case = "copysign_carries_sign"
# subject = "math.copysign"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.copysign: copysign carries the sign bit of the second arg onto the magnitude of the first: copysign(3, -1)==-3.0, copysign(-5, 1)==5.0, and copysign(1, nan) uses NaN's positive sign -> 1.0"""
import math

assert math.copysign(3.0, -1.0) == -3.0, f"copysign(3,-1) = {math.copysign(3.0, -1.0)!r}"
assert math.copysign(-5.0, 1.0) == 5.0, f"copysign(-5,1) = {math.copysign(-5.0, 1.0)!r}"
assert math.copysign(1.0, math.nan) == 1.0, f"copysign(1,nan) = {math.copysign(1.0, math.nan)!r}"

print("copysign_carries_sign OK")
"###);
    assert_output(&out, r###"copysign_carries_sign OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/math/degrees_radians_inverse.py`.
#[test]
fn test_gen_behavior_std_libs_math_degrees_radians_inverse() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "behavior"
# case = "degrees_radians_inverse"
# subject = "math.degrees"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.degrees: degrees and radians are inverses: degrees(pi)==180.0 and radians(180)==pi within 1e-10"""
import math

_eps = 1e-10
assert abs(math.degrees(math.pi) - 180.0) < _eps, f"degrees(pi) = {math.degrees(math.pi)!r}"
assert abs(math.radians(180) - math.pi) < _eps, f"radians(180) = {math.radians(180)!r}"

print("degrees_radians_inverse OK")
"###);
    assert_output(&out, r###"degrees_radians_inverse OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/math/dist_euclidean.py`.
#[test]
fn test_gen_behavior_std_libs_math_dist_euclidean() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "behavior"
# case = "dist_euclidean"
# subject = "math.dist"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.dist: math.dist computes Euclidean distance over equal-length point sequences: dist([0,0],[3,4])==5.0, dist([1,2,3],[4,6,15])==13.0, and accepts tuple inputs"""
import math

assert math.dist([0, 0], [3, 4]) == 5.0, f"dist([0,0],[3,4]) = {math.dist([0, 0], [3, 4])!r}"
assert math.dist([1, 2, 3], [4, 6, 15]) == 13.0, f"dist 3d = {math.dist([1, 2, 3], [4, 6, 15])!r}"
assert math.dist([0], [0]) == 0.0, f"dist([0],[0]) = {math.dist([0], [0])!r}"
assert math.dist((1.5, 2.5), (1.5, 2.5)) == 0.0, "tuple inputs accepted"

print("dist_euclidean OK")
"###);
    assert_output(&out, r###"dist_euclidean OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/math/exp_underflow_to_zero.py`.
#[test]
fn test_gen_behavior_std_libs_math_exp_underflow_to_zero() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "behavior"
# case = "exp_underflow_to_zero"
# subject = "math.exp"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.exp: exp(0)==1.0, exp(1) is e, and exp of a huge negative argument underflows silently to 0.0 (no exception)"""
import math

assert math.exp(0) == 1.0, f"exp(0) = {math.exp(0)!r}"
assert abs(math.exp(1) - math.e) < 1e-10, f"exp(1) = {math.exp(1)!r}"
assert math.exp(-1000000000) == 0.0, f"exp(-1e9) = {math.exp(-1000000000)!r}"

print("exp_underflow_to_zero OK")
"###);
    assert_output(&out, r###"exp_underflow_to_zero OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/math/expm1_log1p_small_arg.py`.
#[test]
fn test_gen_behavior_std_libs_math_expm1_log1p_small_arg() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "behavior"
# case = "expm1_log1p_small_arg"
# subject = "math.expm1"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.expm1: small-arg precision builtins: expm1(0.0)==0.0, expm1(1.0)==1.7182818284590453, log1p(0.0)==0.0, log1p(1.0)==0.6931471805599453"""
import math

assert math.expm1(0.0) == 0.0, f"expm1(0.0) = {math.expm1(0.0)!r}"
assert math.expm1(1.0) == 1.7182818284590453, f"expm1(1.0) = {math.expm1(1.0)!r}"
assert math.log1p(0.0) == 0.0, f"log1p(0.0) = {math.log1p(0.0)!r}"
assert math.log1p(1.0) == 0.6931471805599453, f"log1p(1.0) = {math.log1p(1.0)!r}"

print("expm1_log1p_small_arg OK")
"###);
    assert_output(&out, r###"expm1_log1p_small_arg OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/math/factorial_table.py`.
#[test]
fn test_gen_behavior_std_libs_math_factorial_table() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "behavior"
# case = "factorial_table"
# subject = "math.factorial"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.factorial: math.factorial over a representative table: 0!==1, 1!==1, 5!==120, 10!==3628800, returning int"""
import math

for n, expected in [(0, 1), (1, 1), (5, 120), (10, 3628800)]:
    assert math.factorial(n) == expected, f"{n}! = {math.factorial(n)!r}"
    assert isinstance(math.factorial(n), int), f"{n}! type = {type(math.factorial(n))!r}"

print("factorial_table OK")
"###);
    assert_output(&out, r###"factorial_table OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/math/floor_ceil_return_int.py`.
#[test]
fn test_gen_behavior_std_libs_math_floor_ceil_return_int() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "behavior"
# case = "floor_ceil_return_int"
# subject = "math.floor"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.floor: in Python 3.12 math.floor and math.ceil return int (not float); floor rounds toward -inf, ceil toward +inf, across positive/negative/integral inputs"""
import math

assert isinstance(math.floor(3.7), int), f"floor type = {type(math.floor(3.7))!r}"
assert isinstance(math.ceil(3.2), int), f"ceil type = {type(math.ceil(3.2))!r}"
assert math.floor(3.7) == 3, f"floor(3.7) = {math.floor(3.7)!r}"
assert math.floor(-3.2) == -4, f"floor(-3.2) = {math.floor(-3.2)!r}"
assert math.floor(5.0) == 5, f"floor(5.0) = {math.floor(5.0)!r}"
assert math.ceil(3.2) == 4, f"ceil(3.2) = {math.ceil(3.2)!r}"
assert math.ceil(-3.7) == -3, f"ceil(-3.7) = {math.ceil(-3.7)!r}"
assert math.ceil(5.0) == 5, f"ceil(5.0) = {math.ceil(5.0)!r}"

print("floor_ceil_return_int OK")
"###);
    assert_output(&out, r###"floor_ceil_return_int OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/math/fmod_fabs.py`.
#[test]
fn test_gen_behavior_std_libs_math_fmod_fabs() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "behavior"
# case = "fmod_fabs"
# subject = "math.fmod"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.fmod: fmod keeps the dividend's sign (fmod(-10, 3)==-1.0, fmod(10, 3)==1.0) and fabs returns the absolute value as a float (fabs(-3.14)==3.14)"""
import math

assert math.fmod(10, 3) == 1.0, f"fmod(10,3) = {math.fmod(10, 3)!r}"
assert math.fmod(-10, 3) == -1.0, f"fmod(-10,3) = {math.fmod(-10, 3)!r}"
assert math.fabs(-3.14) == 3.14, f"fabs(-3.14) = {math.fabs(-3.14)!r}"
assert math.fabs(3.14) == 3.14, f"fabs(3.14) = {math.fabs(3.14)!r}"
assert isinstance(math.fabs(-3.14), float), "fabs returns float"

print("fmod_fabs OK")
"###);
    assert_output(&out, r###"fmod_fabs OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/math/fsum_exact_summation.py`.
#[test]
fn test_gen_behavior_std_libs_math_fsum_exact_summation() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "behavior"
# case = "fsum_exact_summation"
# subject = "math.fsum"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.fsum: math.fsum sums ten 0.1 terms to exactly 1.0 where a naive float sum would drift, demonstrating extended-precision (Shewchuk) accumulation"""
import math

# Ten 0.1 terms sum to exactly 1.0 under Shewchuk accumulation.
assert math.fsum([0.1] * 10) == 1.0, f"fsum(10*0.1) = {math.fsum([0.1] * 10)!r}"
# Catastrophic cancellation: the 1e101 terms cancel exactly, leaving 2.0,
# which a single running float accumulator cannot represent mid-sum.
assert math.fsum([1.0, 1e101, 1.0, -1e101]) == 2.0, "fsum cancellation"
# Empty sum is the additive identity 0.0.
assert math.fsum([]) == 0.0, "fsum empty"

print("fsum_exact_summation OK")
"###);
    assert_output(&out, r###"fsum_exact_summation OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/math/gcd_variadic.py`.
#[test]
fn test_gen_behavior_std_libs_math_gcd_variadic() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "behavior"
# case = "gcd_variadic"
# subject = "math.gcd"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.gcd: math.gcd is variadic (CPython 3.9+): gcd()==0, gcd(7)==7, gcd(-12)==12 (abs), gcd(12, 18)==6, gcd(48, 36, 60, 84)==12"""
import math

assert math.gcd() == 0, f"gcd() = {math.gcd()!r}"
assert math.gcd(7) == 7, f"gcd(7) = {math.gcd(7)!r}"
assert math.gcd(-12) == 12, f"gcd(-12) = {math.gcd(-12)!r}"
assert math.gcd(12, 18) == 6, f"gcd(12,18) = {math.gcd(12, 18)!r}"
assert math.gcd(48, 36, 60, 84) == 12, f"gcd(48,36,60,84) = {math.gcd(48, 36, 60, 84)!r}"

print("gcd_variadic OK")
"###);
    assert_output(&out, r###"gcd_variadic OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/math/hypot_variadic.py`.
#[test]
fn test_gen_behavior_std_libs_math_hypot_variadic() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "behavior"
# case = "hypot_variadic"
# subject = "math.hypot"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.hypot: math.hypot is variadic: hypot()==0.0, hypot(-5)==5.0, hypot(3, 4)==5.0 (2-arg), hypot(3, 4, 12)==13.0 (N-arg Euclidean norm)"""
import math

assert math.hypot() == 0.0, f"hypot() = {math.hypot()!r}"
assert math.hypot(-5) == 5.0, f"hypot(-5) = {math.hypot(-5)!r}"
assert math.hypot(5) == 5.0, f"hypot(5) = {math.hypot(5)!r}"
assert math.hypot(3, 4) == 5.0, f"hypot(3,4) = {math.hypot(3, 4)!r}"
assert math.hypot(3, 4, 12) == 13.0, f"hypot(3,4,12) = {math.hypot(3, 4, 12)!r}"

print("hypot_variadic OK")
"###);
    assert_output(&out, r###"hypot_variadic OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/math/inf_comparisons.py`.
#[test]
fn test_gen_behavior_std_libs_math_inf_comparisons() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "behavior"
# case = "inf_comparisons"
# subject = "math.inf"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.inf: math.inf compares as positive infinity (inf > 0, inf == float('inf')) and -math.inf == float('-inf'); nan compares unequal to itself"""
import math

assert math.inf > 0.0, "inf > 0"
assert math.inf == float("inf"), "inf == float('inf')"
assert -math.inf == float("-inf"), "-inf == float('-inf')"
assert math.isinf(math.inf), "inf is inf"
assert math.nan != math.nan, "nan != nan"

print("inf_comparisons OK")
"###);
    assert_output(&out, r###"inf_comparisons OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/math/inverse_trig.py`.
#[test]
fn test_gen_behavior_std_libs_math_inverse_trig() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "behavior"
# case = "inverse_trig"
# subject = "math.atan2"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.atan2: inverse-trig round trips: asin(0)==0, acos(1)==0, atan(0)==0, and atan2(1, 1)==pi/4 within 1e-10"""
import math

_eps = 1e-10
assert abs(math.asin(0) - 0.0) < _eps, f"asin(0) = {math.asin(0)!r}"
assert abs(math.acos(1) - 0.0) < _eps, f"acos(1) = {math.acos(1)!r}"
assert abs(math.atan(0) - 0.0) < _eps, f"atan(0) = {math.atan(0)!r}"
assert abs(math.atan2(1, 1) - math.pi / 4) < _eps, f"atan2(1,1) = {math.atan2(1, 1)!r}"

print("inverse_trig OK")
"###);
    assert_output(&out, r###"inverse_trig OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/math/isnan_isinf_isfinite.py`.
#[test]
fn test_gen_behavior_std_libs_math_isnan_isinf_isfinite() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "behavior"
# case = "isnan_isinf_isfinite"
# subject = "math.isnan"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.isnan: the float predicates classify correctly: isnan(nan) and isnan(float('nan')) are True but isnan(1.0) False; isinf(inf) and isinf(-inf) True; isfinite(1.0) True but isfinite(inf)/isfinite(nan) False"""
import math

assert math.isnan(math.nan), "isnan(nan)"
assert math.isnan(float("nan")), "isnan(float('nan'))"
assert not math.isnan(1.0), "not isnan(1.0)"
assert math.isinf(math.inf), "isinf(inf)"
assert math.isinf(-math.inf), "isinf(-inf)"
assert not math.isinf(1.0), "not isinf(1.0)"
assert math.isfinite(1.0), "isfinite(1.0)"
assert not math.isfinite(math.inf), "not isfinite(inf)"
assert not math.isfinite(math.nan), "not isfinite(nan)"

print("isnan_isinf_isfinite OK")
"###);
    assert_output(&out, r###"isnan_isinf_isfinite OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/math/isqrt_floor_root.py`.
#[test]
fn test_gen_behavior_std_libs_math_isqrt_floor_root() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "behavior"
# case = "isqrt_floor_root"
# subject = "math.isqrt"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.isqrt: math.isqrt returns the integer floor of the square root: isqrt(0)==0, isqrt(99)==9, isqrt(100)==10, isqrt(101)==10, isqrt(10**12)==1000000"""
import math

assert math.isqrt(0) == 0, f"isqrt(0) = {math.isqrt(0)!r}"
assert math.isqrt(99) == 9, f"isqrt(99) = {math.isqrt(99)!r}"
assert math.isqrt(100) == 10, f"isqrt(100) = {math.isqrt(100)!r}"
assert math.isqrt(101) == 10, f"isqrt(101) = {math.isqrt(101)!r}"
assert math.isqrt(10**12) == 1000000, f"isqrt(10**12) = {math.isqrt(10**12)!r}"

print("isqrt_floor_root OK")
"###);
    assert_output(&out, r###"isqrt_floor_root OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/math/lcm_variadic.py`.
#[test]
fn test_gen_behavior_std_libs_math_lcm_variadic() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "behavior"
# case = "lcm_variadic"
# subject = "math.lcm"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.lcm: math.lcm is variadic: lcm()==1, lcm(-7)==7 (abs), lcm(4, 6)==12, lcm(4, 6, 9)==36, lcm(2, 3, 5, 7)==210, and any zero argument short-circuits to 0"""
import math

assert math.lcm() == 1, f"lcm() = {math.lcm()!r}"
assert math.lcm(-7) == 7, f"lcm(-7) = {math.lcm(-7)!r}"
assert math.lcm(4, 6) == 12, f"lcm(4,6) = {math.lcm(4, 6)!r}"
assert math.lcm(4, 6, 9) == 36, f"lcm(4,6,9) = {math.lcm(4, 6, 9)!r}"
assert math.lcm(2, 3, 5, 7) == 210, f"lcm(2,3,5,7) = {math.lcm(2, 3, 5, 7)!r}"
assert math.lcm(4, 0, 9) == 0, f"lcm(4,0,9) = {math.lcm(4, 0, 9)!r}"

print("lcm_variadic OK")
"###);
    assert_output(&out, r###"lcm_variadic OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/math/ldexp_power_of_two.py`.
#[test]
fn test_gen_behavior_std_libs_math_ldexp_power_of_two() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "behavior"
# case = "ldexp_power_of_two"
# subject = "math.ldexp"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.ldexp: math.ldexp(x, i) scales x by 2**i exactly: ldexp(1.5, 3)==12.0, ldexp(1.0, -3)==0.125, ldexp(0.0, 100)==0.0"""
import math

assert math.ldexp(1.5, 3) == 12.0, f"ldexp(1.5,3) = {math.ldexp(1.5, 3)!r}"
assert math.ldexp(1.0, -3) == 0.125, f"ldexp(1.0,-3) = {math.ldexp(1.0, -3)!r}"
assert math.ldexp(0.0, 100) == 0.0, f"ldexp(0.0,100) = {math.ldexp(0.0, 100)!r}"

print("ldexp_power_of_two OK")
"###);
    assert_output(&out, r###"ldexp_power_of_two OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/math/log_family.py`.
#[test]
fn test_gen_behavior_std_libs_math_log_family() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "behavior"
# case = "log_family"
# subject = "math.log"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.log: log(1)==0, log(e)==1, two-arg log(8, 2)==3, log10(1000)==3, log2(8)==3, all within 1e-10"""
import math

_eps = 1e-10
assert abs(math.log(1) - 0.0) < _eps, f"log(1) = {math.log(1)!r}"
assert abs(math.log(math.e) - 1.0) < _eps, f"log(e) = {math.log(math.e)!r}"
assert abs(math.log(8, 2) - 3.0) < _eps, f"log(8,2) = {math.log(8, 2)!r}"
assert abs(math.log10(1000) - 3.0) < _eps, f"log10(1000) = {math.log10(1000)!r}"
assert abs(math.log2(8) - 3.0) < _eps, f"log2(8) = {math.log2(8)!r}"

print("log_family OK")
"###);
    assert_output(&out, r###"log_family OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/math/math_tests__test_inf_constant.py`.
#[test]
fn test_gen_behavior_std_libs_math_math_tests__test_inf_constant() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "behavior"
# case = "math_tests__test_inf_constant"
# subject = "cpython.test_math.MathTests.test_inf_constant"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_math.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_math.py::MathTests::test_inf_constant
"""Auto-ported test: MathTests::test_inf_constant (CPython 3.12 oracle)."""


from test.support import verbose, requires_IEEE_754
from test import support
import unittest
import fractions
import itertools
import decimal
import math
import os
import platform
import random
import struct
import sys


eps = 1e-05

NAN = float('nan')

INF = float('inf')

NINF = float('-inf')

FLOAT_MAX = sys.float_info.max

FLOAT_MIN = sys.float_info.min

x, y = (1e+16, 2.9999)

HAVE_DOUBLE_ROUNDING = x + y == 1e+16 + 4

file = __file__

test_dir = os.path.dirname(file) or os.curdir

math_testcases = os.path.join(test_dir, 'math_testcases.txt')

test_file = os.path.join(test_dir, 'cmath_testcases.txt')

def to_ulps(x):
    """Convert a non-NaN float x to an integer, in such a way that
    adjacent floats are converted to adjacent integers.  Then
    abs(ulps(x) - ulps(y)) gives the difference in ulps between two
    floats.

    The results from this function will only make sense on platforms
    where native doubles are represented in IEEE 754 binary64 format.

    Note: 0.0 and -0.0 are converted to 0 and -1, respectively.
    """
    n = struct.unpack('<q', struct.pack('<d', x))[0]
    if n < 0:
        n = ~(n + 2 ** 63)
    return n

def count_set_bits(n):
    """Number of '1' bits in binary expansion of a nonnnegative integer."""
    return 1 + count_set_bits(n & n - 1) if n else 0

def partial_product(start, stop):
    """Product of integers in range(start, stop, 2), computed recursively.
    start and stop should both be odd, with start <= stop.

    """
    numfactors = stop - start >> 1
    if not numfactors:
        return 1
    elif numfactors == 1:
        return start
    else:
        mid = start + numfactors | 1
        return partial_product(start, mid) * partial_product(mid, stop)

def py_factorial(n):
    """Factorial of nonnegative integer n, via "Binary Split Factorial Formula"
    described at http://www.luschny.de/math/factorial/binarysplitfact.html

    """
    inner = outer = 1
    for i in reversed(range(n.bit_length())):
        inner *= partial_product((n >> i + 1) + 1 | 1, (n >> i) + 1 | 1)
        outer *= inner
    return outer << n - count_set_bits(n)

def ulp_abs_check(expected, got, ulp_tol, abs_tol):
    """Given finite floats `expected` and `got`, check that they're
    approximately equal to within the given number of ulps or the
    given absolute tolerance, whichever is bigger.

    Returns None on success and an error message on failure.
    """
    ulp_error = abs(to_ulps(expected) - to_ulps(got))
    abs_error = abs(expected - got)
    if abs_error <= abs_tol or ulp_error <= ulp_tol:
        return None
    else:
        fmt = 'error = {:.3g} ({:d} ulps); permitted error = {:.3g} or {:d} ulps'
        return fmt.format(abs_error, ulp_error, abs_tol, ulp_tol)

def parse_mtestfile(fname):
    """Parse a file with test values

    -- starts a comment
    blank lines, or lines containing only a comment, are ignored
    other lines are expected to have the form
      id fn arg -> expected [flag]*

    """
    with open(fname, encoding='utf-8') as fp:
        for line in fp:
            if '--' in line:
                line = line[:line.index('--')]
            if not line.strip():
                continue
            lhs, rhs = line.split('->')
            id, fn, arg = lhs.split()
            rhs_pieces = rhs.split()
            exp = rhs_pieces[0]
            flags = rhs_pieces[1:]
            yield (id, fn, float(arg), float(exp), flags)

def parse_testfile(fname):
    """Parse a file with test values

    Empty lines or lines starting with -- are ignored
    yields id, fn, arg_real, arg_imag, exp_real, exp_imag
    """
    with open(fname, encoding='utf-8') as fp:
        for line in fp:
            if line.startswith('--') or not line.strip():
                continue
            lhs, rhs = line.split('->')
            id, fn, arg_real, arg_imag = lhs.split()
            rhs_pieces = rhs.split()
            exp_real, exp_imag = (rhs_pieces[0], rhs_pieces[1])
            flags = rhs_pieces[2:]
            yield (id, fn, float(arg_real), float(arg_imag), float(exp_real), float(exp_imag), flags)

def result_check(expected, got, ulp_tol=5, abs_tol=0.0):
    """Compare arguments expected and got, as floats, if either
    is a float, using a tolerance expressed in multiples of
    ulp(expected) or absolutely (if given and greater).

    As a convenience, when neither argument is a float, and for
    non-finite floats, exact equality is demanded. Also, nan==nan
    as far as this function is concerned.

    Returns None on success and an error message on failure.
    """
    if got == expected:
        if not got and (not expected):
            if math.copysign(1, got) != math.copysign(1, expected):
                return f'expected {expected}, got {got} (zero has wrong sign)'
        return None
    failure = 'not equal'
    if isinstance(expected, float) and isinstance(got, int):
        got = float(got)
    elif isinstance(got, float) and isinstance(expected, int):
        expected = float(expected)
    if isinstance(expected, float) and isinstance(got, float):
        if math.isnan(expected) and math.isnan(got):
            failure = None
        elif math.isinf(expected) or math.isinf(got):
            pass
        else:
            failure = ulp_abs_check(expected, got, ulp_tol, abs_tol)
    if failure is not None:
        fail_fmt = 'expected {!r}, got {!r}'
        fail_msg = fail_fmt.format(expected, got)
        fail_msg += ' ({})'.format(failure)
        return fail_msg
    else:
        return None

class FloatLike:

    def __init__(self, value):
        self.value = value

    def __float__(self):
        return self.value

class IntSubclass(int):
    pass

class MyIndexable(object):

    def __init__(self, value):
        self.value = value

    def __index__(self):
        return self.value

class BadDescr:

    def __get__(self, obj, objtype=None):
        raise ValueError

def load_tests(loader, tests, pattern):
    from doctest import DocFileSuite
    tests.addTest(DocFileSuite('ieee754.txt'))
    return tests


# --- test body ---

assert math.isinf(math.inf)

assert math.inf > 0.0

assert math.inf == float('inf')

assert -math.inf == float('-inf')
print("MathTests::test_inf_constant: ok")
"###);
    assert_output(&out, r###"MathTests::test_inf_constant: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/math/math_tests__test_isfinite.py`.
#[test]
fn test_gen_behavior_std_libs_math_math_tests__test_isfinite() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "behavior"
# case = "math_tests__test_isfinite"
# subject = "cpython.test_math.MathTests.testIsfinite"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_math.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_math.py::MathTests::testIsfinite
"""Auto-ported test: MathTests::testIsfinite (CPython 3.12 oracle)."""


from test.support import verbose, requires_IEEE_754
from test import support
import unittest
import fractions
import itertools
import decimal
import math
import os
import platform
import random
import struct
import sys


eps = 1e-05

NAN = float('nan')

INF = float('inf')

NINF = float('-inf')

FLOAT_MAX = sys.float_info.max

FLOAT_MIN = sys.float_info.min

x, y = (1e+16, 2.9999)

HAVE_DOUBLE_ROUNDING = x + y == 1e+16 + 4

file = __file__

test_dir = os.path.dirname(file) or os.curdir

math_testcases = os.path.join(test_dir, 'math_testcases.txt')

test_file = os.path.join(test_dir, 'cmath_testcases.txt')

def to_ulps(x):
    """Convert a non-NaN float x to an integer, in such a way that
    adjacent floats are converted to adjacent integers.  Then
    abs(ulps(x) - ulps(y)) gives the difference in ulps between two
    floats.

    The results from this function will only make sense on platforms
    where native doubles are represented in IEEE 754 binary64 format.

    Note: 0.0 and -0.0 are converted to 0 and -1, respectively.
    """
    n = struct.unpack('<q', struct.pack('<d', x))[0]
    if n < 0:
        n = ~(n + 2 ** 63)
    return n

def count_set_bits(n):
    """Number of '1' bits in binary expansion of a nonnnegative integer."""
    return 1 + count_set_bits(n & n - 1) if n else 0

def partial_product(start, stop):
    """Product of integers in range(start, stop, 2), computed recursively.
    start and stop should both be odd, with start <= stop.

    """
    numfactors = stop - start >> 1
    if not numfactors:
        return 1
    elif numfactors == 1:
        return start
    else:
        mid = start + numfactors | 1
        return partial_product(start, mid) * partial_product(mid, stop)

def py_factorial(n):
    """Factorial of nonnegative integer n, via "Binary Split Factorial Formula"
    described at http://www.luschny.de/math/factorial/binarysplitfact.html

    """
    inner = outer = 1
    for i in reversed(range(n.bit_length())):
        inner *= partial_product((n >> i + 1) + 1 | 1, (n >> i) + 1 | 1)
        outer *= inner
    return outer << n - count_set_bits(n)

def ulp_abs_check(expected, got, ulp_tol, abs_tol):
    """Given finite floats `expected` and `got`, check that they're
    approximately equal to within the given number of ulps or the
    given absolute tolerance, whichever is bigger.

    Returns None on success and an error message on failure.
    """
    ulp_error = abs(to_ulps(expected) - to_ulps(got))
    abs_error = abs(expected - got)
    if abs_error <= abs_tol or ulp_error <= ulp_tol:
        return None
    else:
        fmt = 'error = {:.3g} ({:d} ulps); permitted error = {:.3g} or {:d} ulps'
        return fmt.format(abs_error, ulp_error, abs_tol, ulp_tol)

def parse_mtestfile(fname):
    """Parse a file with test values

    -- starts a comment
    blank lines, or lines containing only a comment, are ignored
    other lines are expected to have the form
      id fn arg -> expected [flag]*

    """
    with open(fname, encoding='utf-8') as fp:
        for line in fp:
            if '--' in line:
                line = line[:line.index('--')]
            if not line.strip():
                continue
            lhs, rhs = line.split('->')
            id, fn, arg = lhs.split()
            rhs_pieces = rhs.split()
            exp = rhs_pieces[0]
            flags = rhs_pieces[1:]
            yield (id, fn, float(arg), float(exp), flags)

def parse_testfile(fname):
    """Parse a file with test values

    Empty lines or lines starting with -- are ignored
    yields id, fn, arg_real, arg_imag, exp_real, exp_imag
    """
    with open(fname, encoding='utf-8') as fp:
        for line in fp:
            if line.startswith('--') or not line.strip():
                continue
            lhs, rhs = line.split('->')
            id, fn, arg_real, arg_imag = lhs.split()
            rhs_pieces = rhs.split()
            exp_real, exp_imag = (rhs_pieces[0], rhs_pieces[1])
            flags = rhs_pieces[2:]
            yield (id, fn, float(arg_real), float(arg_imag), float(exp_real), float(exp_imag), flags)

def result_check(expected, got, ulp_tol=5, abs_tol=0.0):
    """Compare arguments expected and got, as floats, if either
    is a float, using a tolerance expressed in multiples of
    ulp(expected) or absolutely (if given and greater).

    As a convenience, when neither argument is a float, and for
    non-finite floats, exact equality is demanded. Also, nan==nan
    as far as this function is concerned.

    Returns None on success and an error message on failure.
    """
    if got == expected:
        if not got and (not expected):
            if math.copysign(1, got) != math.copysign(1, expected):
                return f'expected {expected}, got {got} (zero has wrong sign)'
        return None
    failure = 'not equal'
    if isinstance(expected, float) and isinstance(got, int):
        got = float(got)
    elif isinstance(got, float) and isinstance(expected, int):
        expected = float(expected)
    if isinstance(expected, float) and isinstance(got, float):
        if math.isnan(expected) and math.isnan(got):
            failure = None
        elif math.isinf(expected) or math.isinf(got):
            pass
        else:
            failure = ulp_abs_check(expected, got, ulp_tol, abs_tol)
    if failure is not None:
        fail_fmt = 'expected {!r}, got {!r}'
        fail_msg = fail_fmt.format(expected, got)
        fail_msg += ' ({})'.format(failure)
        return fail_msg
    else:
        return None

class FloatLike:

    def __init__(self, value):
        self.value = value

    def __float__(self):
        return self.value

class IntSubclass(int):
    pass

class MyIndexable(object):

    def __init__(self, value):
        self.value = value

    def __index__(self):
        return self.value

class BadDescr:

    def __get__(self, obj, objtype=None):
        raise ValueError

def load_tests(loader, tests, pattern):
    from doctest import DocFileSuite
    tests.addTest(DocFileSuite('ieee754.txt'))
    return tests


# --- test body ---
def assertEqualSign(x, y):
    """Similar to assertEqual(), but compare also the sign with copysign().

        Function useful to compare signed zeros.
        """

    assert x == y

    assert math.copysign(1.0, x) == math.copysign(1.0, y)

def assertIsNaN(value):
    if not math.isnan(value):

        raise AssertionError('Expected a NaN, got {!r}.'.format(value))

def ftest(name, got, expected, ulp_tol=5, abs_tol=0.0):
    """Compare arguments expected and got, as floats, if either
        is a float, using a tolerance expressed in multiples of
        ulp(expected) or absolutely, whichever is greater.

        As a convenience, when neither argument is a float, and for
        non-finite floats, exact equality is demanded. Also, nan==nan
        in this function.
        """
    failure = result_check(expected, got, ulp_tol, abs_tol)
    if failure is not None:

        raise AssertionError('{}: {}'.format(name, failure))

assert math.isfinite(0.0)

assert math.isfinite(-0.0)

assert math.isfinite(1.0)

assert math.isfinite(-1.0)

assert not math.isfinite(float('nan'))

assert not math.isfinite(float('inf'))

assert not math.isfinite(float('-inf'))
print("MathTests::testIsfinite: ok")
"###);
    assert_output(&out, r###"MathTests::testIsfinite: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/math/math_tests__test_isinf.py`.
#[test]
fn test_gen_behavior_std_libs_math_math_tests__test_isinf() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "behavior"
# case = "math_tests__test_isinf"
# subject = "cpython.test_math.MathTests.testIsinf"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_math.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_math.py::MathTests::testIsinf
"""Auto-ported test: MathTests::testIsinf (CPython 3.12 oracle)."""


from test.support import verbose, requires_IEEE_754
from test import support
import unittest
import fractions
import itertools
import decimal
import math
import os
import platform
import random
import struct
import sys


eps = 1e-05

NAN = float('nan')

INF = float('inf')

NINF = float('-inf')

FLOAT_MAX = sys.float_info.max

FLOAT_MIN = sys.float_info.min

x, y = (1e+16, 2.9999)

HAVE_DOUBLE_ROUNDING = x + y == 1e+16 + 4

file = __file__

test_dir = os.path.dirname(file) or os.curdir

math_testcases = os.path.join(test_dir, 'math_testcases.txt')

test_file = os.path.join(test_dir, 'cmath_testcases.txt')

def to_ulps(x):
    """Convert a non-NaN float x to an integer, in such a way that
    adjacent floats are converted to adjacent integers.  Then
    abs(ulps(x) - ulps(y)) gives the difference in ulps between two
    floats.

    The results from this function will only make sense on platforms
    where native doubles are represented in IEEE 754 binary64 format.

    Note: 0.0 and -0.0 are converted to 0 and -1, respectively.
    """
    n = struct.unpack('<q', struct.pack('<d', x))[0]
    if n < 0:
        n = ~(n + 2 ** 63)
    return n

def count_set_bits(n):
    """Number of '1' bits in binary expansion of a nonnnegative integer."""
    return 1 + count_set_bits(n & n - 1) if n else 0

def partial_product(start, stop):
    """Product of integers in range(start, stop, 2), computed recursively.
    start and stop should both be odd, with start <= stop.

    """
    numfactors = stop - start >> 1
    if not numfactors:
        return 1
    elif numfactors == 1:
        return start
    else:
        mid = start + numfactors | 1
        return partial_product(start, mid) * partial_product(mid, stop)

def py_factorial(n):
    """Factorial of nonnegative integer n, via "Binary Split Factorial Formula"
    described at http://www.luschny.de/math/factorial/binarysplitfact.html

    """
    inner = outer = 1
    for i in reversed(range(n.bit_length())):
        inner *= partial_product((n >> i + 1) + 1 | 1, (n >> i) + 1 | 1)
        outer *= inner
    return outer << n - count_set_bits(n)

def ulp_abs_check(expected, got, ulp_tol, abs_tol):
    """Given finite floats `expected` and `got`, check that they're
    approximately equal to within the given number of ulps or the
    given absolute tolerance, whichever is bigger.

    Returns None on success and an error message on failure.
    """
    ulp_error = abs(to_ulps(expected) - to_ulps(got))
    abs_error = abs(expected - got)
    if abs_error <= abs_tol or ulp_error <= ulp_tol:
        return None
    else:
        fmt = 'error = {:.3g} ({:d} ulps); permitted error = {:.3g} or {:d} ulps'
        return fmt.format(abs_error, ulp_error, abs_tol, ulp_tol)

def parse_mtestfile(fname):
    """Parse a file with test values

    -- starts a comment
    blank lines, or lines containing only a comment, are ignored
    other lines are expected to have the form
      id fn arg -> expected [flag]*

    """
    with open(fname, encoding='utf-8') as fp:
        for line in fp:
            if '--' in line:
                line = line[:line.index('--')]
            if not line.strip():
                continue
            lhs, rhs = line.split('->')
            id, fn, arg = lhs.split()
            rhs_pieces = rhs.split()
            exp = rhs_pieces[0]
            flags = rhs_pieces[1:]
            yield (id, fn, float(arg), float(exp), flags)

def parse_testfile(fname):
    """Parse a file with test values

    Empty lines or lines starting with -- are ignored
    yields id, fn, arg_real, arg_imag, exp_real, exp_imag
    """
    with open(fname, encoding='utf-8') as fp:
        for line in fp:
            if line.startswith('--') or not line.strip():
                continue
            lhs, rhs = line.split('->')
            id, fn, arg_real, arg_imag = lhs.split()
            rhs_pieces = rhs.split()
            exp_real, exp_imag = (rhs_pieces[0], rhs_pieces[1])
            flags = rhs_pieces[2:]
            yield (id, fn, float(arg_real), float(arg_imag), float(exp_real), float(exp_imag), flags)

def result_check(expected, got, ulp_tol=5, abs_tol=0.0):
    """Compare arguments expected and got, as floats, if either
    is a float, using a tolerance expressed in multiples of
    ulp(expected) or absolutely (if given and greater).

    As a convenience, when neither argument is a float, and for
    non-finite floats, exact equality is demanded. Also, nan==nan
    as far as this function is concerned.

    Returns None on success and an error message on failure.
    """
    if got == expected:
        if not got and (not expected):
            if math.copysign(1, got) != math.copysign(1, expected):
                return f'expected {expected}, got {got} (zero has wrong sign)'
        return None
    failure = 'not equal'
    if isinstance(expected, float) and isinstance(got, int):
        got = float(got)
    elif isinstance(got, float) and isinstance(expected, int):
        expected = float(expected)
    if isinstance(expected, float) and isinstance(got, float):
        if math.isnan(expected) and math.isnan(got):
            failure = None
        elif math.isinf(expected) or math.isinf(got):
            pass
        else:
            failure = ulp_abs_check(expected, got, ulp_tol, abs_tol)
    if failure is not None:
        fail_fmt = 'expected {!r}, got {!r}'
        fail_msg = fail_fmt.format(expected, got)
        fail_msg += ' ({})'.format(failure)
        return fail_msg
    else:
        return None

class FloatLike:

    def __init__(self, value):
        self.value = value

    def __float__(self):
        return self.value

class IntSubclass(int):
    pass

class MyIndexable(object):

    def __init__(self, value):
        self.value = value

    def __index__(self):
        return self.value

class BadDescr:

    def __get__(self, obj, objtype=None):
        raise ValueError

def load_tests(loader, tests, pattern):
    from doctest import DocFileSuite
    tests.addTest(DocFileSuite('ieee754.txt'))
    return tests


# --- test body ---
def assertEqualSign(x, y):
    """Similar to assertEqual(), but compare also the sign with copysign().

        Function useful to compare signed zeros.
        """

    assert x == y

    assert math.copysign(1.0, x) == math.copysign(1.0, y)

def assertIsNaN(value):
    if not math.isnan(value):

        raise AssertionError('Expected a NaN, got {!r}.'.format(value))

def ftest(name, got, expected, ulp_tol=5, abs_tol=0.0):
    """Compare arguments expected and got, as floats, if either
        is a float, using a tolerance expressed in multiples of
        ulp(expected) or absolutely, whichever is greater.

        As a convenience, when neither argument is a float, and for
        non-finite floats, exact equality is demanded. Also, nan==nan
        in this function.
        """
    failure = result_check(expected, got, ulp_tol, abs_tol)
    if failure is not None:

        raise AssertionError('{}: {}'.format(name, failure))

assert math.isinf(float('inf'))

assert math.isinf(float('-inf'))

assert math.isinf(1e309)

assert math.isinf(-1e309)

assert not math.isinf(float('nan'))

assert not math.isinf(0.0)

assert not math.isinf(1.0)
print("MathTests::testIsinf: ok")
"###);
    assert_output(&out, r###"MathTests::testIsinf: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/math/math_tests__test_isnan.py`.
#[test]
fn test_gen_behavior_std_libs_math_math_tests__test_isnan() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "behavior"
# case = "math_tests__test_isnan"
# subject = "cpython.test_math.MathTests.testIsnan"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_math.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_math.py::MathTests::testIsnan
"""Auto-ported test: MathTests::testIsnan (CPython 3.12 oracle)."""


from test.support import verbose, requires_IEEE_754
from test import support
import unittest
import fractions
import itertools
import decimal
import math
import os
import platform
import random
import struct
import sys


eps = 1e-05

NAN = float('nan')

INF = float('inf')

NINF = float('-inf')

FLOAT_MAX = sys.float_info.max

FLOAT_MIN = sys.float_info.min

x, y = (1e+16, 2.9999)

HAVE_DOUBLE_ROUNDING = x + y == 1e+16 + 4

file = __file__

test_dir = os.path.dirname(file) or os.curdir

math_testcases = os.path.join(test_dir, 'math_testcases.txt')

test_file = os.path.join(test_dir, 'cmath_testcases.txt')

def to_ulps(x):
    """Convert a non-NaN float x to an integer, in such a way that
    adjacent floats are converted to adjacent integers.  Then
    abs(ulps(x) - ulps(y)) gives the difference in ulps between two
    floats.

    The results from this function will only make sense on platforms
    where native doubles are represented in IEEE 754 binary64 format.

    Note: 0.0 and -0.0 are converted to 0 and -1, respectively.
    """
    n = struct.unpack('<q', struct.pack('<d', x))[0]
    if n < 0:
        n = ~(n + 2 ** 63)
    return n

def count_set_bits(n):
    """Number of '1' bits in binary expansion of a nonnnegative integer."""
    return 1 + count_set_bits(n & n - 1) if n else 0

def partial_product(start, stop):
    """Product of integers in range(start, stop, 2), computed recursively.
    start and stop should both be odd, with start <= stop.

    """
    numfactors = stop - start >> 1
    if not numfactors:
        return 1
    elif numfactors == 1:
        return start
    else:
        mid = start + numfactors | 1
        return partial_product(start, mid) * partial_product(mid, stop)

def py_factorial(n):
    """Factorial of nonnegative integer n, via "Binary Split Factorial Formula"
    described at http://www.luschny.de/math/factorial/binarysplitfact.html

    """
    inner = outer = 1
    for i in reversed(range(n.bit_length())):
        inner *= partial_product((n >> i + 1) + 1 | 1, (n >> i) + 1 | 1)
        outer *= inner
    return outer << n - count_set_bits(n)

def ulp_abs_check(expected, got, ulp_tol, abs_tol):
    """Given finite floats `expected` and `got`, check that they're
    approximately equal to within the given number of ulps or the
    given absolute tolerance, whichever is bigger.

    Returns None on success and an error message on failure.
    """
    ulp_error = abs(to_ulps(expected) - to_ulps(got))
    abs_error = abs(expected - got)
    if abs_error <= abs_tol or ulp_error <= ulp_tol:
        return None
    else:
        fmt = 'error = {:.3g} ({:d} ulps); permitted error = {:.3g} or {:d} ulps'
        return fmt.format(abs_error, ulp_error, abs_tol, ulp_tol)

def parse_mtestfile(fname):
    """Parse a file with test values

    -- starts a comment
    blank lines, or lines containing only a comment, are ignored
    other lines are expected to have the form
      id fn arg -> expected [flag]*

    """
    with open(fname, encoding='utf-8') as fp:
        for line in fp:
            if '--' in line:
                line = line[:line.index('--')]
            if not line.strip():
                continue
            lhs, rhs = line.split('->')
            id, fn, arg = lhs.split()
            rhs_pieces = rhs.split()
            exp = rhs_pieces[0]
            flags = rhs_pieces[1:]
            yield (id, fn, float(arg), float(exp), flags)

def parse_testfile(fname):
    """Parse a file with test values

    Empty lines or lines starting with -- are ignored
    yields id, fn, arg_real, arg_imag, exp_real, exp_imag
    """
    with open(fname, encoding='utf-8') as fp:
        for line in fp:
            if line.startswith('--') or not line.strip():
                continue
            lhs, rhs = line.split('->')
            id, fn, arg_real, arg_imag = lhs.split()
            rhs_pieces = rhs.split()
            exp_real, exp_imag = (rhs_pieces[0], rhs_pieces[1])
            flags = rhs_pieces[2:]
            yield (id, fn, float(arg_real), float(arg_imag), float(exp_real), float(exp_imag), flags)

def result_check(expected, got, ulp_tol=5, abs_tol=0.0):
    """Compare arguments expected and got, as floats, if either
    is a float, using a tolerance expressed in multiples of
    ulp(expected) or absolutely (if given and greater).

    As a convenience, when neither argument is a float, and for
    non-finite floats, exact equality is demanded. Also, nan==nan
    as far as this function is concerned.

    Returns None on success and an error message on failure.
    """
    if got == expected:
        if not got and (not expected):
            if math.copysign(1, got) != math.copysign(1, expected):
                return f'expected {expected}, got {got} (zero has wrong sign)'
        return None
    failure = 'not equal'
    if isinstance(expected, float) and isinstance(got, int):
        got = float(got)
    elif isinstance(got, float) and isinstance(expected, int):
        expected = float(expected)
    if isinstance(expected, float) and isinstance(got, float):
        if math.isnan(expected) and math.isnan(got):
            failure = None
        elif math.isinf(expected) or math.isinf(got):
            pass
        else:
            failure = ulp_abs_check(expected, got, ulp_tol, abs_tol)
    if failure is not None:
        fail_fmt = 'expected {!r}, got {!r}'
        fail_msg = fail_fmt.format(expected, got)
        fail_msg += ' ({})'.format(failure)
        return fail_msg
    else:
        return None

class FloatLike:

    def __init__(self, value):
        self.value = value

    def __float__(self):
        return self.value

class IntSubclass(int):
    pass

class MyIndexable(object):

    def __init__(self, value):
        self.value = value

    def __index__(self):
        return self.value

class BadDescr:

    def __get__(self, obj, objtype=None):
        raise ValueError

def load_tests(loader, tests, pattern):
    from doctest import DocFileSuite
    tests.addTest(DocFileSuite('ieee754.txt'))
    return tests


# --- test body ---
def assertEqualSign(x, y):
    """Similar to assertEqual(), but compare also the sign with copysign().

        Function useful to compare signed zeros.
        """

    assert x == y

    assert math.copysign(1.0, x) == math.copysign(1.0, y)

def assertIsNaN(value):
    if not math.isnan(value):

        raise AssertionError('Expected a NaN, got {!r}.'.format(value))

def ftest(name, got, expected, ulp_tol=5, abs_tol=0.0):
    """Compare arguments expected and got, as floats, if either
        is a float, using a tolerance expressed in multiples of
        ulp(expected) or absolutely, whichever is greater.

        As a convenience, when neither argument is a float, and for
        non-finite floats, exact equality is demanded. Also, nan==nan
        in this function.
        """
    failure = result_check(expected, got, ulp_tol, abs_tol)
    if failure is not None:

        raise AssertionError('{}: {}'.format(name, failure))

assert math.isnan(float('nan'))

assert math.isnan(float('-nan'))

assert math.isnan(float('inf') * 0.0)

assert not math.isnan(float('inf'))

assert not math.isnan(0.0)

assert not math.isnan(1.0)
print("MathTests::testIsnan: ok")
"###);
    assert_output(&out, r###"MathTests::testIsnan: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/math/math_tests__test_issue39871.py`.
#[test]
fn test_gen_behavior_std_libs_math_math_tests__test_issue39871() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "behavior"
# case = "math_tests__test_issue39871"
# subject = "cpython.test_math.MathTests.test_issue39871"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_math.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_math.py::MathTests::test_issue39871
"""Auto-ported test: MathTests::test_issue39871 (CPython 3.12 oracle)."""


from test.support import verbose, requires_IEEE_754
from test import support
import unittest
import fractions
import itertools
import decimal
import math
import os
import platform
import random
import struct
import sys


eps = 1e-05

NAN = float('nan')

INF = float('inf')

NINF = float('-inf')

FLOAT_MAX = sys.float_info.max

FLOAT_MIN = sys.float_info.min

x, y = (1e+16, 2.9999)

HAVE_DOUBLE_ROUNDING = x + y == 1e+16 + 4

file = __file__

test_dir = os.path.dirname(file) or os.curdir

math_testcases = os.path.join(test_dir, 'math_testcases.txt')

test_file = os.path.join(test_dir, 'cmath_testcases.txt')

def to_ulps(x):
    """Convert a non-NaN float x to an integer, in such a way that
    adjacent floats are converted to adjacent integers.  Then
    abs(ulps(x) - ulps(y)) gives the difference in ulps between two
    floats.

    The results from this function will only make sense on platforms
    where native doubles are represented in IEEE 754 binary64 format.

    Note: 0.0 and -0.0 are converted to 0 and -1, respectively.
    """
    n = struct.unpack('<q', struct.pack('<d', x))[0]
    if n < 0:
        n = ~(n + 2 ** 63)
    return n

def count_set_bits(n):
    """Number of '1' bits in binary expansion of a nonnnegative integer."""
    return 1 + count_set_bits(n & n - 1) if n else 0

def partial_product(start, stop):
    """Product of integers in range(start, stop, 2), computed recursively.
    start and stop should both be odd, with start <= stop.

    """
    numfactors = stop - start >> 1
    if not numfactors:
        return 1
    elif numfactors == 1:
        return start
    else:
        mid = start + numfactors | 1
        return partial_product(start, mid) * partial_product(mid, stop)

def py_factorial(n):
    """Factorial of nonnegative integer n, via "Binary Split Factorial Formula"
    described at http://www.luschny.de/math/factorial/binarysplitfact.html

    """
    inner = outer = 1
    for i in reversed(range(n.bit_length())):
        inner *= partial_product((n >> i + 1) + 1 | 1, (n >> i) + 1 | 1)
        outer *= inner
    return outer << n - count_set_bits(n)

def ulp_abs_check(expected, got, ulp_tol, abs_tol):
    """Given finite floats `expected` and `got`, check that they're
    approximately equal to within the given number of ulps or the
    given absolute tolerance, whichever is bigger.

    Returns None on success and an error message on failure.
    """
    ulp_error = abs(to_ulps(expected) - to_ulps(got))
    abs_error = abs(expected - got)
    if abs_error <= abs_tol or ulp_error <= ulp_tol:
        return None
    else:
        fmt = 'error = {:.3g} ({:d} ulps); permitted error = {:.3g} or {:d} ulps'
        return fmt.format(abs_error, ulp_error, abs_tol, ulp_tol)

def parse_mtestfile(fname):
    """Parse a file with test values

    -- starts a comment
    blank lines, or lines containing only a comment, are ignored
    other lines are expected to have the form
      id fn arg -> expected [flag]*

    """
    with open(fname, encoding='utf-8') as fp:
        for line in fp:
            if '--' in line:
                line = line[:line.index('--')]
            if not line.strip():
                continue
            lhs, rhs = line.split('->')
            id, fn, arg = lhs.split()
            rhs_pieces = rhs.split()
            exp = rhs_pieces[0]
            flags = rhs_pieces[1:]
            yield (id, fn, float(arg), float(exp), flags)

def parse_testfile(fname):
    """Parse a file with test values

    Empty lines or lines starting with -- are ignored
    yields id, fn, arg_real, arg_imag, exp_real, exp_imag
    """
    with open(fname, encoding='utf-8') as fp:
        for line in fp:
            if line.startswith('--') or not line.strip():
                continue
            lhs, rhs = line.split('->')
            id, fn, arg_real, arg_imag = lhs.split()
            rhs_pieces = rhs.split()
            exp_real, exp_imag = (rhs_pieces[0], rhs_pieces[1])
            flags = rhs_pieces[2:]
            yield (id, fn, float(arg_real), float(arg_imag), float(exp_real), float(exp_imag), flags)

def result_check(expected, got, ulp_tol=5, abs_tol=0.0):
    """Compare arguments expected and got, as floats, if either
    is a float, using a tolerance expressed in multiples of
    ulp(expected) or absolutely (if given and greater).

    As a convenience, when neither argument is a float, and for
    non-finite floats, exact equality is demanded. Also, nan==nan
    as far as this function is concerned.

    Returns None on success and an error message on failure.
    """
    if got == expected:
        if not got and (not expected):
            if math.copysign(1, got) != math.copysign(1, expected):
                return f'expected {expected}, got {got} (zero has wrong sign)'
        return None
    failure = 'not equal'
    if isinstance(expected, float) and isinstance(got, int):
        got = float(got)
    elif isinstance(got, float) and isinstance(expected, int):
        expected = float(expected)
    if isinstance(expected, float) and isinstance(got, float):
        if math.isnan(expected) and math.isnan(got):
            failure = None
        elif math.isinf(expected) or math.isinf(got):
            pass
        else:
            failure = ulp_abs_check(expected, got, ulp_tol, abs_tol)
    if failure is not None:
        fail_fmt = 'expected {!r}, got {!r}'
        fail_msg = fail_fmt.format(expected, got)
        fail_msg += ' ({})'.format(failure)
        return fail_msg
    else:
        return None

class FloatLike:

    def __init__(self, value):
        self.value = value

    def __float__(self):
        return self.value

class IntSubclass(int):
    pass

class MyIndexable(object):

    def __init__(self, value):
        self.value = value

    def __index__(self):
        return self.value

class BadDescr:

    def __get__(self, obj, objtype=None):
        raise ValueError

def load_tests(loader, tests, pattern):
    from doctest import DocFileSuite
    tests.addTest(DocFileSuite('ieee754.txt'))
    return tests


# --- test body ---
class F:

    def __float__(self):
        self.converted = True
        1 / 0
for func in (math.atan2, math.copysign, math.remainder):
    y = F()
    try:
        func('not a number', y)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

    assert not getattr(y, 'converted', False)
print("MathTests::test_issue39871: ok")
"###);
    assert_output(&out, r###"MathTests::test_issue39871: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/math/math_tests__test_math_dist_leak.py`.
#[test]
fn test_gen_behavior_std_libs_math_math_tests__test_math_dist_leak() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "behavior"
# case = "math_tests__test_math_dist_leak"
# subject = "cpython.test_math.MathTests.test_math_dist_leak"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_math.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_math.py::MathTests::test_math_dist_leak
"""Auto-ported test: MathTests::test_math_dist_leak (CPython 3.12 oracle)."""


from test.support import verbose, requires_IEEE_754
from test import support
import unittest
import fractions
import itertools
import decimal
import math
import os
import platform
import random
import struct
import sys


eps = 1e-05

NAN = float('nan')

INF = float('inf')

NINF = float('-inf')

FLOAT_MAX = sys.float_info.max

FLOAT_MIN = sys.float_info.min

x, y = (1e+16, 2.9999)

HAVE_DOUBLE_ROUNDING = x + y == 1e+16 + 4

file = __file__

test_dir = os.path.dirname(file) or os.curdir

math_testcases = os.path.join(test_dir, 'math_testcases.txt')

test_file = os.path.join(test_dir, 'cmath_testcases.txt')

def to_ulps(x):
    """Convert a non-NaN float x to an integer, in such a way that
    adjacent floats are converted to adjacent integers.  Then
    abs(ulps(x) - ulps(y)) gives the difference in ulps between two
    floats.

    The results from this function will only make sense on platforms
    where native doubles are represented in IEEE 754 binary64 format.

    Note: 0.0 and -0.0 are converted to 0 and -1, respectively.
    """
    n = struct.unpack('<q', struct.pack('<d', x))[0]
    if n < 0:
        n = ~(n + 2 ** 63)
    return n

def count_set_bits(n):
    """Number of '1' bits in binary expansion of a nonnnegative integer."""
    return 1 + count_set_bits(n & n - 1) if n else 0

def partial_product(start, stop):
    """Product of integers in range(start, stop, 2), computed recursively.
    start and stop should both be odd, with start <= stop.

    """
    numfactors = stop - start >> 1
    if not numfactors:
        return 1
    elif numfactors == 1:
        return start
    else:
        mid = start + numfactors | 1
        return partial_product(start, mid) * partial_product(mid, stop)

def py_factorial(n):
    """Factorial of nonnegative integer n, via "Binary Split Factorial Formula"
    described at http://www.luschny.de/math/factorial/binarysplitfact.html

    """
    inner = outer = 1
    for i in reversed(range(n.bit_length())):
        inner *= partial_product((n >> i + 1) + 1 | 1, (n >> i) + 1 | 1)
        outer *= inner
    return outer << n - count_set_bits(n)

def ulp_abs_check(expected, got, ulp_tol, abs_tol):
    """Given finite floats `expected` and `got`, check that they're
    approximately equal to within the given number of ulps or the
    given absolute tolerance, whichever is bigger.

    Returns None on success and an error message on failure.
    """
    ulp_error = abs(to_ulps(expected) - to_ulps(got))
    abs_error = abs(expected - got)
    if abs_error <= abs_tol or ulp_error <= ulp_tol:
        return None
    else:
        fmt = 'error = {:.3g} ({:d} ulps); permitted error = {:.3g} or {:d} ulps'
        return fmt.format(abs_error, ulp_error, abs_tol, ulp_tol)

def parse_mtestfile(fname):
    """Parse a file with test values

    -- starts a comment
    blank lines, or lines containing only a comment, are ignored
    other lines are expected to have the form
      id fn arg -> expected [flag]*

    """
    with open(fname, encoding='utf-8') as fp:
        for line in fp:
            if '--' in line:
                line = line[:line.index('--')]
            if not line.strip():
                continue
            lhs, rhs = line.split('->')
            id, fn, arg = lhs.split()
            rhs_pieces = rhs.split()
            exp = rhs_pieces[0]
            flags = rhs_pieces[1:]
            yield (id, fn, float(arg), float(exp), flags)

def parse_testfile(fname):
    """Parse a file with test values

    Empty lines or lines starting with -- are ignored
    yields id, fn, arg_real, arg_imag, exp_real, exp_imag
    """
    with open(fname, encoding='utf-8') as fp:
        for line in fp:
            if line.startswith('--') or not line.strip():
                continue
            lhs, rhs = line.split('->')
            id, fn, arg_real, arg_imag = lhs.split()
            rhs_pieces = rhs.split()
            exp_real, exp_imag = (rhs_pieces[0], rhs_pieces[1])
            flags = rhs_pieces[2:]
            yield (id, fn, float(arg_real), float(arg_imag), float(exp_real), float(exp_imag), flags)

def result_check(expected, got, ulp_tol=5, abs_tol=0.0):
    """Compare arguments expected and got, as floats, if either
    is a float, using a tolerance expressed in multiples of
    ulp(expected) or absolutely (if given and greater).

    As a convenience, when neither argument is a float, and for
    non-finite floats, exact equality is demanded. Also, nan==nan
    as far as this function is concerned.

    Returns None on success and an error message on failure.
    """
    if got == expected:
        if not got and (not expected):
            if math.copysign(1, got) != math.copysign(1, expected):
                return f'expected {expected}, got {got} (zero has wrong sign)'
        return None
    failure = 'not equal'
    if isinstance(expected, float) and isinstance(got, int):
        got = float(got)
    elif isinstance(got, float) and isinstance(expected, int):
        expected = float(expected)
    if isinstance(expected, float) and isinstance(got, float):
        if math.isnan(expected) and math.isnan(got):
            failure = None
        elif math.isinf(expected) or math.isinf(got):
            pass
        else:
            failure = ulp_abs_check(expected, got, ulp_tol, abs_tol)
    if failure is not None:
        fail_fmt = 'expected {!r}, got {!r}'
        fail_msg = fail_fmt.format(expected, got)
        fail_msg += ' ({})'.format(failure)
        return fail_msg
    else:
        return None

class FloatLike:

    def __init__(self, value):
        self.value = value

    def __float__(self):
        return self.value

class IntSubclass(int):
    pass

class MyIndexable(object):

    def __init__(self, value):
        self.value = value

    def __index__(self):
        return self.value

class BadDescr:

    def __get__(self, obj, objtype=None):
        raise ValueError

def load_tests(loader, tests, pattern):
    from doctest import DocFileSuite
    tests.addTest(DocFileSuite('ieee754.txt'))
    return tests


# --- test body ---
try:
    math.dist([1, 2], [3, 4, 5])
    raise AssertionError('expected ValueError')
except ValueError:
    pass
print("MathTests::test_math_dist_leak: ok")
"###);
    assert_output(&out, r###"MathTests::test_math_dist_leak: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/math/math_tests__test_nan_constant.py`.
#[test]
fn test_gen_behavior_std_libs_math_math_tests__test_nan_constant() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "behavior"
# case = "math_tests__test_nan_constant"
# subject = "cpython.test_math.MathTests.test_nan_constant"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_math.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_math.py::MathTests::test_nan_constant
"""Auto-ported test: MathTests::test_nan_constant (CPython 3.12 oracle)."""


from test.support import verbose, requires_IEEE_754
from test import support
import unittest
import fractions
import itertools
import decimal
import math
import os
import platform
import random
import struct
import sys


eps = 1e-05

NAN = float('nan')

INF = float('inf')

NINF = float('-inf')

FLOAT_MAX = sys.float_info.max

FLOAT_MIN = sys.float_info.min

x, y = (1e+16, 2.9999)

HAVE_DOUBLE_ROUNDING = x + y == 1e+16 + 4

file = __file__

test_dir = os.path.dirname(file) or os.curdir

math_testcases = os.path.join(test_dir, 'math_testcases.txt')

test_file = os.path.join(test_dir, 'cmath_testcases.txt')

def to_ulps(x):
    """Convert a non-NaN float x to an integer, in such a way that
    adjacent floats are converted to adjacent integers.  Then
    abs(ulps(x) - ulps(y)) gives the difference in ulps between two
    floats.

    The results from this function will only make sense on platforms
    where native doubles are represented in IEEE 754 binary64 format.

    Note: 0.0 and -0.0 are converted to 0 and -1, respectively.
    """
    n = struct.unpack('<q', struct.pack('<d', x))[0]
    if n < 0:
        n = ~(n + 2 ** 63)
    return n

def count_set_bits(n):
    """Number of '1' bits in binary expansion of a nonnnegative integer."""
    return 1 + count_set_bits(n & n - 1) if n else 0

def partial_product(start, stop):
    """Product of integers in range(start, stop, 2), computed recursively.
    start and stop should both be odd, with start <= stop.

    """
    numfactors = stop - start >> 1
    if not numfactors:
        return 1
    elif numfactors == 1:
        return start
    else:
        mid = start + numfactors | 1
        return partial_product(start, mid) * partial_product(mid, stop)

def py_factorial(n):
    """Factorial of nonnegative integer n, via "Binary Split Factorial Formula"
    described at http://www.luschny.de/math/factorial/binarysplitfact.html

    """
    inner = outer = 1
    for i in reversed(range(n.bit_length())):
        inner *= partial_product((n >> i + 1) + 1 | 1, (n >> i) + 1 | 1)
        outer *= inner
    return outer << n - count_set_bits(n)

def ulp_abs_check(expected, got, ulp_tol, abs_tol):
    """Given finite floats `expected` and `got`, check that they're
    approximately equal to within the given number of ulps or the
    given absolute tolerance, whichever is bigger.

    Returns None on success and an error message on failure.
    """
    ulp_error = abs(to_ulps(expected) - to_ulps(got))
    abs_error = abs(expected - got)
    if abs_error <= abs_tol or ulp_error <= ulp_tol:
        return None
    else:
        fmt = 'error = {:.3g} ({:d} ulps); permitted error = {:.3g} or {:d} ulps'
        return fmt.format(abs_error, ulp_error, abs_tol, ulp_tol)

def parse_mtestfile(fname):
    """Parse a file with test values

    -- starts a comment
    blank lines, or lines containing only a comment, are ignored
    other lines are expected to have the form
      id fn arg -> expected [flag]*

    """
    with open(fname, encoding='utf-8') as fp:
        for line in fp:
            if '--' in line:
                line = line[:line.index('--')]
            if not line.strip():
                continue
            lhs, rhs = line.split('->')
            id, fn, arg = lhs.split()
            rhs_pieces = rhs.split()
            exp = rhs_pieces[0]
            flags = rhs_pieces[1:]
            yield (id, fn, float(arg), float(exp), flags)

def parse_testfile(fname):
    """Parse a file with test values

    Empty lines or lines starting with -- are ignored
    yields id, fn, arg_real, arg_imag, exp_real, exp_imag
    """
    with open(fname, encoding='utf-8') as fp:
        for line in fp:
            if line.startswith('--') or not line.strip():
                continue
            lhs, rhs = line.split('->')
            id, fn, arg_real, arg_imag = lhs.split()
            rhs_pieces = rhs.split()
            exp_real, exp_imag = (rhs_pieces[0], rhs_pieces[1])
            flags = rhs_pieces[2:]
            yield (id, fn, float(arg_real), float(arg_imag), float(exp_real), float(exp_imag), flags)

def result_check(expected, got, ulp_tol=5, abs_tol=0.0):
    """Compare arguments expected and got, as floats, if either
    is a float, using a tolerance expressed in multiples of
    ulp(expected) or absolutely (if given and greater).

    As a convenience, when neither argument is a float, and for
    non-finite floats, exact equality is demanded. Also, nan==nan
    as far as this function is concerned.

    Returns None on success and an error message on failure.
    """
    if got == expected:
        if not got and (not expected):
            if math.copysign(1, got) != math.copysign(1, expected):
                return f'expected {expected}, got {got} (zero has wrong sign)'
        return None
    failure = 'not equal'
    if isinstance(expected, float) and isinstance(got, int):
        got = float(got)
    elif isinstance(got, float) and isinstance(expected, int):
        expected = float(expected)
    if isinstance(expected, float) and isinstance(got, float):
        if math.isnan(expected) and math.isnan(got):
            failure = None
        elif math.isinf(expected) or math.isinf(got):
            pass
        else:
            failure = ulp_abs_check(expected, got, ulp_tol, abs_tol)
    if failure is not None:
        fail_fmt = 'expected {!r}, got {!r}'
        fail_msg = fail_fmt.format(expected, got)
        fail_msg += ' ({})'.format(failure)
        return fail_msg
    else:
        return None

class FloatLike:

    def __init__(self, value):
        self.value = value

    def __float__(self):
        return self.value

class IntSubclass(int):
    pass

class MyIndexable(object):

    def __init__(self, value):
        self.value = value

    def __index__(self):
        return self.value

class BadDescr:

    def __get__(self, obj, objtype=None):
        raise ValueError

def load_tests(loader, tests, pattern):
    from doctest import DocFileSuite
    tests.addTest(DocFileSuite('ieee754.txt'))
    return tests


# --- test body ---

assert math.isnan(math.nan)

assert math.copysign(1.0, math.nan) == 1.0
print("MathTests::test_nan_constant: ok")
"###);
    assert_output(&out, r###"MathTests::test_nan_constant: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/math/math_tests__test_tanh_sign.py`.
#[test]
fn test_gen_behavior_std_libs_math_math_tests__test_tanh_sign() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "behavior"
# case = "math_tests__test_tanh_sign"
# subject = "cpython.test_math.MathTests.testTanhSign"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_math.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_math.py::MathTests::testTanhSign
"""Auto-ported test: MathTests::testTanhSign (CPython 3.12 oracle)."""


from test.support import verbose, requires_IEEE_754
from test import support
import unittest
import fractions
import itertools
import decimal
import math
import os
import platform
import random
import struct
import sys


eps = 1e-05

NAN = float('nan')

INF = float('inf')

NINF = float('-inf')

FLOAT_MAX = sys.float_info.max

FLOAT_MIN = sys.float_info.min

x, y = (1e+16, 2.9999)

HAVE_DOUBLE_ROUNDING = x + y == 1e+16 + 4

file = __file__

test_dir = os.path.dirname(file) or os.curdir

math_testcases = os.path.join(test_dir, 'math_testcases.txt')

test_file = os.path.join(test_dir, 'cmath_testcases.txt')

def to_ulps(x):
    """Convert a non-NaN float x to an integer, in such a way that
    adjacent floats are converted to adjacent integers.  Then
    abs(ulps(x) - ulps(y)) gives the difference in ulps between two
    floats.

    The results from this function will only make sense on platforms
    where native doubles are represented in IEEE 754 binary64 format.

    Note: 0.0 and -0.0 are converted to 0 and -1, respectively.
    """
    n = struct.unpack('<q', struct.pack('<d', x))[0]
    if n < 0:
        n = ~(n + 2 ** 63)
    return n

def count_set_bits(n):
    """Number of '1' bits in binary expansion of a nonnnegative integer."""
    return 1 + count_set_bits(n & n - 1) if n else 0

def partial_product(start, stop):
    """Product of integers in range(start, stop, 2), computed recursively.
    start and stop should both be odd, with start <= stop.

    """
    numfactors = stop - start >> 1
    if not numfactors:
        return 1
    elif numfactors == 1:
        return start
    else:
        mid = start + numfactors | 1
        return partial_product(start, mid) * partial_product(mid, stop)

def py_factorial(n):
    """Factorial of nonnegative integer n, via "Binary Split Factorial Formula"
    described at http://www.luschny.de/math/factorial/binarysplitfact.html

    """
    inner = outer = 1
    for i in reversed(range(n.bit_length())):
        inner *= partial_product((n >> i + 1) + 1 | 1, (n >> i) + 1 | 1)
        outer *= inner
    return outer << n - count_set_bits(n)

def ulp_abs_check(expected, got, ulp_tol, abs_tol):
    """Given finite floats `expected` and `got`, check that they're
    approximately equal to within the given number of ulps or the
    given absolute tolerance, whichever is bigger.

    Returns None on success and an error message on failure.
    """
    ulp_error = abs(to_ulps(expected) - to_ulps(got))
    abs_error = abs(expected - got)
    if abs_error <= abs_tol or ulp_error <= ulp_tol:
        return None
    else:
        fmt = 'error = {:.3g} ({:d} ulps); permitted error = {:.3g} or {:d} ulps'
        return fmt.format(abs_error, ulp_error, abs_tol, ulp_tol)

def parse_mtestfile(fname):
    """Parse a file with test values

    -- starts a comment
    blank lines, or lines containing only a comment, are ignored
    other lines are expected to have the form
      id fn arg -> expected [flag]*

    """
    with open(fname, encoding='utf-8') as fp:
        for line in fp:
            if '--' in line:
                line = line[:line.index('--')]
            if not line.strip():
                continue
            lhs, rhs = line.split('->')
            id, fn, arg = lhs.split()
            rhs_pieces = rhs.split()
            exp = rhs_pieces[0]
            flags = rhs_pieces[1:]
            yield (id, fn, float(arg), float(exp), flags)

def parse_testfile(fname):
    """Parse a file with test values

    Empty lines or lines starting with -- are ignored
    yields id, fn, arg_real, arg_imag, exp_real, exp_imag
    """
    with open(fname, encoding='utf-8') as fp:
        for line in fp:
            if line.startswith('--') or not line.strip():
                continue
            lhs, rhs = line.split('->')
            id, fn, arg_real, arg_imag = lhs.split()
            rhs_pieces = rhs.split()
            exp_real, exp_imag = (rhs_pieces[0], rhs_pieces[1])
            flags = rhs_pieces[2:]
            yield (id, fn, float(arg_real), float(arg_imag), float(exp_real), float(exp_imag), flags)

def result_check(expected, got, ulp_tol=5, abs_tol=0.0):
    """Compare arguments expected and got, as floats, if either
    is a float, using a tolerance expressed in multiples of
    ulp(expected) or absolutely (if given and greater).

    As a convenience, when neither argument is a float, and for
    non-finite floats, exact equality is demanded. Also, nan==nan
    as far as this function is concerned.

    Returns None on success and an error message on failure.
    """
    if got == expected:
        if not got and (not expected):
            if math.copysign(1, got) != math.copysign(1, expected):
                return f'expected {expected}, got {got} (zero has wrong sign)'
        return None
    failure = 'not equal'
    if isinstance(expected, float) and isinstance(got, int):
        got = float(got)
    elif isinstance(got, float) and isinstance(expected, int):
        expected = float(expected)
    if isinstance(expected, float) and isinstance(got, float):
        if math.isnan(expected) and math.isnan(got):
            failure = None
        elif math.isinf(expected) or math.isinf(got):
            pass
        else:
            failure = ulp_abs_check(expected, got, ulp_tol, abs_tol)
    if failure is not None:
        fail_fmt = 'expected {!r}, got {!r}'
        fail_msg = fail_fmt.format(expected, got)
        fail_msg += ' ({})'.format(failure)
        return fail_msg
    else:
        return None

class FloatLike:

    def __init__(self, value):
        self.value = value

    def __float__(self):
        return self.value

class IntSubclass(int):
    pass

class MyIndexable(object):

    def __init__(self, value):
        self.value = value

    def __index__(self):
        return self.value

class BadDescr:

    def __get__(self, obj, objtype=None):
        raise ValueError

def load_tests(loader, tests, pattern):
    from doctest import DocFileSuite
    tests.addTest(DocFileSuite('ieee754.txt'))
    return tests


# --- test body ---
def assertEqualSign(x, y):
    """Similar to assertEqual(), but compare also the sign with copysign().

        Function useful to compare signed zeros.
        """

    assert x == y

    assert math.copysign(1.0, x) == math.copysign(1.0, y)

def assertIsNaN(value):
    if not math.isnan(value):

        raise AssertionError('Expected a NaN, got {!r}.'.format(value))

def ftest(name, got, expected, ulp_tol=5, abs_tol=0.0):
    """Compare arguments expected and got, as floats, if either
        is a float, using a tolerance expressed in multiples of
        ulp(expected) or absolutely, whichever is greater.

        As a convenience, when neither argument is a float, and for
        non-finite floats, exact equality is demanded. Also, nan==nan
        in this function.
        """
    failure = result_check(expected, got, ulp_tol, abs_tol)
    if failure is not None:

        raise AssertionError('{}: {}'.format(name, failure))

assert math.tanh(-0.0) == -0.0

assert math.copysign(1.0, math.tanh(-0.0)) == math.copysign(1.0, -0.0)
print("MathTests::testTanhSign: ok")
"###);
    assert_output(&out, r###"MathTests::testTanhSign: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/math/modf_frexp_decomposition.py`.
#[test]
fn test_gen_behavior_std_libs_math_modf_frexp_decomposition() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "behavior"
# case = "modf_frexp_decomposition"
# subject = "math.modf"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.modf: modf(3.75) splits into (0.75, 3.0) fractional/integral float parts and frexp(8.0) returns the (mantissa, exponent) pair (0.5, 4)"""
import math

frac_part, int_part = math.modf(3.75)
assert frac_part == 0.75, f"modf frac = {frac_part!r}"
assert int_part == 3.0, f"modf int = {int_part!r}"
assert isinstance(frac_part, float) and isinstance(int_part, float), "modf parts are float"
m, e = math.frexp(8.0)
assert m == 0.5, f"frexp mantissa = {m!r}"
assert e == 4, f"frexp exponent = {e!r}"

print("modf_frexp_decomposition OK")
"###);
    assert_output(&out, r###"modf_frexp_decomposition OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/math/pow_returns_float.py`.
#[test]
fn test_gen_behavior_std_libs_math_pow_returns_float() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "behavior"
# case = "pow_returns_float"
# subject = "math.pow"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.pow: math.pow always returns a float: pow(2, 10)==1024.0, pow(5, 0)==1.0, pow(2, -1)==0.5, pow(2, 0.5) is sqrt(2)"""
import math

assert math.pow(2, 10) == 1024.0, f"pow(2,10) = {math.pow(2, 10)!r}"
assert isinstance(math.pow(2, 10), float), "pow returns float"
assert math.pow(5, 0) == 1.0, f"pow(5,0) = {math.pow(5, 0)!r}"
assert math.pow(2, -1) == 0.5, f"pow(2,-1) = {math.pow(2, -1)!r}"
assert abs(math.pow(2, 0.5) - math.sqrt(2)) < 1e-10, f"pow(2,0.5) = {math.pow(2, 0.5)!r}"

print("pow_returns_float OK")
"###);
    assert_output(&out, r###"pow_returns_float OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/math/remainder_round_half_even.py`.
#[test]
fn test_gen_behavior_std_libs_math_remainder_round_half_even() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "behavior"
# case = "remainder_round_half_even"
# subject = "math.remainder"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.remainder: IEEE-754 remainder rounds the quotient to nearest-even: remainder(7, 3)==1.0, remainder(7.5, 5)==-2.5, remainder(-7, 3)==-1.0, remainder(10, 4)==2.0"""
import math

assert math.remainder(7, 3) == 1.0, f"remainder(7,3) = {math.remainder(7, 3)!r}"
assert math.remainder(7.5, 5) == -2.5, f"remainder(7.5,5) = {math.remainder(7.5, 5)!r}"
assert math.remainder(-7, 3) == -1.0, f"remainder(-7,3) = {math.remainder(-7, 3)!r}"
assert math.remainder(10, 4) == 2.0, f"remainder(10,4) = {math.remainder(10, 4)!r}"

print("remainder_round_half_even OK")
"###);
    assert_output(&out, r###"remainder_round_half_even OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/math/sqrt_returns_float.py`.
#[test]
fn test_gen_behavior_std_libs_math_sqrt_returns_float() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "behavior"
# case = "sqrt_returns_float"
# subject = "math.sqrt"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.sqrt: math.sqrt always returns a float (sqrt(4) is 2.0, not int 2) and sqrt(2)==1.4142135623730951; sqrt(0)==0.0, sqrt(1)==1.0"""
import math

assert isinstance(math.sqrt(4), float), f"sqrt type = {type(math.sqrt(4))!r}"
assert math.sqrt(4) == 2.0, f"sqrt(4) = {math.sqrt(4)!r}"
assert math.sqrt(2) == 1.4142135623730951, f"sqrt(2) = {math.sqrt(2)!r}"
assert math.sqrt(0) == 0.0, f"sqrt(0) = {math.sqrt(0)!r}"
assert math.sqrt(1) == 1.0, f"sqrt(1) = {math.sqrt(1)!r}"

print("sqrt_returns_float OK")
"###);
    assert_output(&out, r###"sqrt_returns_float OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/math/sumprod_dot_product.py`.
#[test]
fn test_gen_behavior_std_libs_math_sumprod_dot_product() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "behavior"
# case = "sumprod_dot_product"
# subject = "math.sumprod"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.sumprod: math.sumprod is an extended-precision dot product: int operands stay int (sumprod([1,2,3],[4,5,6])==32), empty operands give 0, bool coerces like 0/1, and catastrophic 1e101 cancellation leaves 2.0 exactly"""
import math

assert math.sumprod([1, 2, 3], [4, 5, 6]) == 32, "sumprod ints"
assert isinstance(math.sumprod([1, 2, 3], [4, 5, 6]), int), "int result type"
assert math.sumprod([], []) == 0, "sumprod empty"
assert math.sumprod([0.1] * 20, [True, False] * 10) == 1.0, "bool second arg"
assert math.sumprod([True, False] * 10, [0.1] * 20) == 1.0, "bool first arg"
got = math.sumprod([1.0, 1e101, 1.0, -1e101], [1.0] * 4)
assert got == 2.0, f"cancellation = {got!r}"

print("sumprod_dot_product OK")
"###);
    assert_output(&out, r###"sumprod_dot_product OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/math/trig_identities.py`.
#[test]
fn test_gen_behavior_std_libs_math_trig_identities() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "behavior"
# case = "trig_identities"
# subject = "math.sin"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.sin: core trig identities within 1e-10: sin(pi)~=0, cos(pi)==-1, tan(pi/4)==1, sin(0)==0, cos(0)==1, sin(pi/2)==1"""
import math

_eps = 1e-10
assert abs(math.sin(math.pi) - 0.0) < _eps, f"sin(pi) = {math.sin(math.pi)!r}"
assert abs(math.cos(math.pi) - (-1.0)) < _eps, f"cos(pi) = {math.cos(math.pi)!r}"
assert abs(math.tan(math.pi / 4) - 1.0) < _eps, f"tan(pi/4) = {math.tan(math.pi / 4)!r}"
assert abs(math.sin(0)) < _eps, f"sin(0) = {math.sin(0)!r}"
assert abs(math.cos(0) - 1.0) < _eps, f"cos(0) = {math.cos(0)!r}"
assert abs(math.sin(math.pi / 2) - 1.0) < _eps, f"sin(pi/2) = {math.sin(math.pi / 2)!r}"

print("trig_identities OK")
"###);
    assert_output(&out, r###"trig_identities OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/math/trunc_toward_zero.py`.
#[test]
fn test_gen_behavior_std_libs_math_trunc_toward_zero() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "behavior"
# case = "trunc_toward_zero"
# subject = "math.trunc"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.trunc: math.trunc rounds toward zero and returns int: trunc(3.7)==3, trunc(-3.7)==-3, trunc(0.5)==0"""
import math

assert math.trunc(3.7) == 3, f"trunc(3.7) = {math.trunc(3.7)!r}"
assert math.trunc(-3.7) == -3, f"trunc(-3.7) = {math.trunc(-3.7)!r}"
assert math.trunc(0.5) == 0, f"trunc(0.5) = {math.trunc(0.5)!r}"
assert isinstance(math.trunc(3.7), int), f"trunc type = {type(math.trunc(3.7))!r}"

print("trunc_toward_zero OK")
"###);
    assert_output(&out, r###"trunc_toward_zero OK
"###);
}

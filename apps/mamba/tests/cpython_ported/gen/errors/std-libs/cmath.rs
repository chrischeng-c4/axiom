use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/cmath/abs_overflow_raises_overflowerror.py`.
#[test]
fn test_gen_errors_std_libs_cmath_abs_overflow_raises_overflowerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "errors"
# case = "abs_overflow_raises_overflowerror"
# subject = "abs"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_cmath.py"
# status = "filled"
# ///
"""abs: abs_overflow_raises_overflowerror (errors)."""
import cmath  # noqa: F401

_raised = False
try:
    abs(complex(1.4e308, 1.4e308))
except OverflowError:
    _raised = True
assert _raised, "abs_overflow_raises_overflowerror: expected OverflowError"
print("abs_overflow_raises_overflowerror OK")
"###);
    assert_output(&out, r###"abs_overflow_raises_overflowerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/cmath/exp_huge_raises_overflowerror.py`.
#[test]
fn test_gen_errors_std_libs_cmath_exp_huge_raises_overflowerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "errors"
# case = "exp_huge_raises_overflowerror"
# subject = "cmath.exp"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_cmath.py"
# status = "filled"
# ///
"""cmath.exp: exp_huge_raises_overflowerror (errors)."""
import cmath

_raised = False
try:
    cmath.exp(1e9)
except OverflowError:
    _raised = True
assert _raised, "exp_huge_raises_overflowerror: expected OverflowError"
print("exp_huge_raises_overflowerror OK")
"###);
    assert_output(&out, r###"exp_huge_raises_overflowerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/cmath/isclose_complex_abs_tol_raises_typeerror.py`.
#[test]
fn test_gen_errors_std_libs_cmath_isclose_complex_abs_tol_raises_typeerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "errors"
# case = "isclose_complex_abs_tol_raises_typeerror"
# subject = "cmath.isclose"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_cmath.py"
# status = "filled"
# ///
"""cmath.isclose: isclose_complex_abs_tol_raises_typeerror (errors)."""
import cmath

_raised = False
try:
    cmath.isclose(1j, 1j, abs_tol=1j)
except TypeError:
    _raised = True
assert _raised, "isclose_complex_abs_tol_raises_typeerror: expected TypeError"
print("isclose_complex_abs_tol_raises_typeerror OK")
"###);
    assert_output(&out, r###"isclose_complex_abs_tol_raises_typeerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/cmath/isclose_complex_rel_tol_raises_typeerror.py`.
#[test]
fn test_gen_errors_std_libs_cmath_isclose_complex_rel_tol_raises_typeerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "errors"
# case = "isclose_complex_rel_tol_raises_typeerror"
# subject = "cmath.isclose"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_cmath.py"
# status = "filled"
# ///
"""cmath.isclose: isclose_complex_rel_tol_raises_typeerror (errors)."""
import cmath

_raised = False
try:
    cmath.isclose(1j, 1j, rel_tol=1j)
except TypeError:
    _raised = True
assert _raised, "isclose_complex_rel_tol_raises_typeerror: expected TypeError"
print("isclose_complex_rel_tol_raises_typeerror OK")
"###);
    assert_output(&out, r###"isclose_complex_rel_tol_raises_typeerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/cmath/isclose_negative_rel_tol_raises_valueerror.py`.
#[test]
fn test_gen_errors_std_libs_cmath_isclose_negative_rel_tol_raises_valueerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "errors"
# case = "isclose_negative_rel_tol_raises_valueerror"
# subject = "cmath.isclose"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_cmath.py"
# status = "filled"
# ///
"""cmath.isclose: isclose_negative_rel_tol_raises_valueerror (errors)."""
import cmath

_raised = False
try:
    cmath.isclose(1.0, 1.0, rel_tol=-1.0)
except ValueError:
    _raised = True
assert _raised, "isclose_negative_rel_tol_raises_valueerror: expected ValueError"
print("isclose_negative_rel_tol_raises_valueerror OK")
"###);
    assert_output(&out, r###"isclose_negative_rel_tol_raises_valueerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/cmath/log_zero_raises_valueerror.py`.
#[test]
fn test_gen_errors_std_libs_cmath_log_zero_raises_valueerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "errors"
# case = "log_zero_raises_valueerror"
# subject = "cmath.log"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_cmath.py"
# status = "filled"
# ///
"""cmath.log: log_zero_raises_valueerror (errors)."""
import cmath

_raised = False
try:
    cmath.log(0)
except ValueError:
    _raised = True
assert _raised, "log_zero_raises_valueerror: expected ValueError"
print("log_zero_raises_valueerror OK")
"###);
    assert_output(&out, r###"log_zero_raises_valueerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/cmath/sqrt_str_raises_typeerror.py`.
#[test]
fn test_gen_errors_std_libs_cmath_sqrt_str_raises_typeerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "errors"
# case = "sqrt_str_raises_typeerror"
# subject = "cmath.sqrt"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_cmath.py"
# status = "filled"
# ///
"""cmath.sqrt: sqrt_str_raises_typeerror (errors)."""
import cmath

_raised = False
try:
    cmath.sqrt("hello")
except TypeError:
    _raised = True
assert _raised, "sqrt_str_raises_typeerror: expected TypeError"
print("sqrt_str_raises_typeerror OK")
"###);
    assert_output(&out, r###"sqrt_str_raises_typeerror OK
"###);
}

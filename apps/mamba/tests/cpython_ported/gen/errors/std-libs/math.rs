use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/math/atan2_bad_first_typeerror.py`.
#[test]
fn test_gen_errors_std_libs_math_atan2_bad_first_typeerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "errors"
# case = "atan2_bad_first_typeerror"
# subject = "math.atan2"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.atan2: atan2_bad_first_typeerror (errors)."""
import math

_raised = False
try:
    math.atan2("spam", 1.0)
except TypeError:
    _raised = True
assert _raised, "atan2_bad_first_typeerror: expected TypeError"
print("atan2_bad_first_typeerror OK")
"###);
    assert_output(&out, r###"atan2_bad_first_typeerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/math/atan2_bad_second_typeerror.py`.
#[test]
fn test_gen_errors_std_libs_math_atan2_bad_second_typeerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "errors"
# case = "atan2_bad_second_typeerror"
# subject = "math.atan2"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.atan2: atan2_bad_second_typeerror (errors)."""
import math

_raised = False
try:
    math.atan2(1.0, "spam")
except TypeError:
    _raised = True
assert _raised, "atan2_bad_second_typeerror: expected TypeError"
print("atan2_bad_second_typeerror OK")
"###);
    assert_output(&out, r###"atan2_bad_second_typeerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/math/binary_funcs_reject_bad_first_before_second.py`.
#[test]
fn test_gen_errors_std_libs_math_binary_funcs_reject_bad_first_before_second() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "errors"
# case = "binary_funcs_reject_bad_first_before_second"
# subject = "math.atan2"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.atan2: atan2/copysign/remainder validate the first operand's type before touching the second; a bad first arg raises TypeError without ever calling __float__ on the second operand (bpo-39871 regression)"""
import math

class Tripwire:
    """Raises if __float__ is ever invoked, recording the attempt."""

    def __init__(self):
        self.converted = False

    def __float__(self):
        self.converted = True
        raise ZeroDivisionError("__float__ should not have been called")


for func in (math.atan2, math.copysign, math.remainder):
    probe = Tripwire()
    _raised = False
    try:
        func("not a number", probe)
    except TypeError:
        _raised = True
    assert _raised, f"{func.__name__}: bad first arg raises TypeError"
    assert not probe.converted, f"{func.__name__}: second arg left untouched"

print("binary_funcs_reject_bad_first_before_second OK")
"###);
    assert_output(&out, r###"binary_funcs_reject_bad_first_before_second OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/math/dist_length_mismatch_valueerror.py`.
#[test]
fn test_gen_errors_std_libs_math_dist_length_mismatch_valueerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "errors"
# case = "dist_length_mismatch_valueerror"
# subject = "math.dist"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.dist: dist_length_mismatch_valueerror (errors)."""
import math

_raised = False
try:
    math.dist([1, 2], [3, 4, 5])
except ValueError:
    _raised = True
assert _raised, "dist_length_mismatch_valueerror: expected ValueError"
print("dist_length_mismatch_valueerror OK")
"###);
    assert_output(&out, r###"dist_length_mismatch_valueerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/math/erf_str_typeerror.py`.
#[test]
fn test_gen_errors_std_libs_math_erf_str_typeerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "errors"
# case = "erf_str_typeerror"
# subject = "math.erf"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.erf: erf_str_typeerror (errors)."""
import math

_raised = False
try:
    math.erf("spam")
except TypeError:
    _raised = True
assert _raised, "erf_str_typeerror: expected TypeError"
print("erf_str_typeerror OK")
"###);
    assert_output(&out, r###"erf_str_typeerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/math/exp_overflow_overflowerror.py`.
#[test]
fn test_gen_errors_std_libs_math_exp_overflow_overflowerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "errors"
# case = "exp_overflow_overflowerror"
# subject = "math.exp"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.exp: exp_overflow_overflowerror (errors)."""
import math

_raised = False
try:
    math.exp(1e9)
except OverflowError:
    _raised = True
assert _raised, "exp_overflow_overflowerror: expected OverflowError"
print("exp_overflow_overflowerror OK")
"###);
    assert_output(&out, r###"exp_overflow_overflowerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/math/factorial_negative_valueerror.py`.
#[test]
fn test_gen_errors_std_libs_math_factorial_negative_valueerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "errors"
# case = "factorial_negative_valueerror"
# subject = "math.factorial"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.factorial: factorial_negative_valueerror (errors)."""
import math

_raised = False
try:
    math.factorial(-1)
except ValueError:
    _raised = True
assert _raised, "factorial_negative_valueerror: expected ValueError"
print("factorial_negative_valueerror OK")
"###);
    assert_output(&out, r###"factorial_negative_valueerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/math/isqrt_negative_valueerror.py`.
#[test]
fn test_gen_errors_std_libs_math_isqrt_negative_valueerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "errors"
# case = "isqrt_negative_valueerror"
# subject = "math.isqrt"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.isqrt: isqrt_negative_valueerror (errors)."""
import math

_raised = False
try:
    math.isqrt(-1)
except ValueError:
    _raised = True
assert _raised, "isqrt_negative_valueerror: expected ValueError"
print("isqrt_negative_valueerror OK")
"###);
    assert_output(&out, r###"isqrt_negative_valueerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/math/log_negative_valueerror.py`.
#[test]
fn test_gen_errors_std_libs_math_log_negative_valueerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "errors"
# case = "log_negative_valueerror"
# subject = "math.log"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.log: log_negative_valueerror (errors)."""
import math

_raised = False
try:
    math.log(-1)
except ValueError:
    _raised = True
assert _raised, "log_negative_valueerror: expected ValueError"
print("log_negative_valueerror OK")
"###);
    assert_output(&out, r###"log_negative_valueerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/math/log_zero_valueerror.py`.
#[test]
fn test_gen_errors_std_libs_math_log_zero_valueerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "errors"
# case = "log_zero_valueerror"
# subject = "math.log"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.log: log_zero_valueerror (errors)."""
import math

_raised = False
try:
    math.log(0)
except ValueError:
    _raised = True
assert _raised, "log_zero_valueerror: expected ValueError"
print("log_zero_valueerror OK")
"###);
    assert_output(&out, r###"log_zero_valueerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/math/sqrt_negative_valueerror.py`.
#[test]
fn test_gen_errors_std_libs_math_sqrt_negative_valueerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "errors"
# case = "sqrt_negative_valueerror"
# subject = "math.sqrt"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.sqrt: sqrt_negative_valueerror (errors)."""
import math

_raised = False
try:
    math.sqrt(-1)
except ValueError:
    _raised = True
assert _raised, "sqrt_negative_valueerror: expected ValueError"
print("sqrt_negative_valueerror OK")
"###);
    assert_output(&out, r###"sqrt_negative_valueerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/math/sqrt_str_typeerror.py`.
#[test]
fn test_gen_errors_std_libs_math_sqrt_str_typeerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "errors"
# case = "sqrt_str_typeerror"
# subject = "math.sqrt"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.sqrt: sqrt_str_typeerror (errors)."""
import math

_raised = False
try:
    math.sqrt("hello")
except TypeError:
    _raised = True
assert _raised, "sqrt_str_typeerror: expected TypeError"
print("sqrt_str_typeerror OK")
"###);
    assert_output(&out, r###"sqrt_str_typeerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/math/sumprod_length_mismatch_valueerror.py`.
#[test]
fn test_gen_errors_std_libs_math_sumprod_length_mismatch_valueerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "errors"
# case = "sumprod_length_mismatch_valueerror"
# subject = "math.sumprod"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.sumprod: sumprod_length_mismatch_valueerror (errors)."""
import math

_raised = False
try:
    math.sumprod([1, 2], [3])
except ValueError:
    _raised = True
assert _raised, "sumprod_length_mismatch_valueerror: expected ValueError"
print("sumprod_length_mismatch_valueerror OK")
"###);
    assert_output(&out, r###"sumprod_length_mismatch_valueerror OK
"###);
}

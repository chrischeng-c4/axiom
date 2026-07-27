use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/fractions/bad_string_raises.py`.
#[test]
fn test_gen_errors_std_libs_fractions_bad_string_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fractions"
# dimension = "errors"
# case = "bad_string_raises"
# subject = "fractions.Fraction"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fractions.py"
# status = "filled"
# ///
"""fractions.Fraction: bad_string_raises (errors)."""
import fractions

_raised = False
try:
    fractions.Fraction('not_a_number')
except ValueError:
    _raised = True
assert _raised, "bad_string_raises: expected ValueError"
print("bad_string_raises OK")
"###);
    assert_output(&out, r###"bad_string_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/fractions/divide_by_zero_fraction_raises.py`.
#[test]
fn test_gen_errors_std_libs_fractions_divide_by_zero_fraction_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fractions"
# dimension = "errors"
# case = "divide_by_zero_fraction_raises"
# subject = "fractions.Fraction"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fractions.py"
# status = "filled"
# ///
"""fractions.Fraction: divide_by_zero_fraction_raises (errors)."""
import fractions

_raised = False
try:
    fractions.Fraction(1, 2) / fractions.Fraction(0)
except ZeroDivisionError:
    _raised = True
assert _raised, "divide_by_zero_fraction_raises: expected ZeroDivisionError"
print("divide_by_zero_fraction_raises OK")
"###);
    assert_output(&out, r###"divide_by_zero_fraction_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/fractions/float_inf_raises.py`.
#[test]
fn test_gen_errors_std_libs_fractions_float_inf_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fractions"
# dimension = "errors"
# case = "float_inf_raises"
# subject = "fractions.Fraction"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fractions.py"
# status = "filled"
# ///
"""fractions.Fraction: float_inf_raises (errors)."""
import fractions

_raised = False
try:
    fractions.Fraction(float('inf'))
except OverflowError:
    _raised = True
assert _raised, "float_inf_raises: expected OverflowError"
print("float_inf_raises OK")
"###);
    assert_output(&out, r###"float_inf_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/fractions/float_nan_raises.py`.
#[test]
fn test_gen_errors_std_libs_fractions_float_nan_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fractions"
# dimension = "errors"
# case = "float_nan_raises"
# subject = "fractions.Fraction"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fractions.py"
# status = "filled"
# ///
"""fractions.Fraction: float_nan_raises (errors)."""
import fractions

_raised = False
try:
    fractions.Fraction(float('nan'))
except ValueError:
    _raised = True
assert _raised, "float_nan_raises: expected ValueError"
print("float_nan_raises OK")
"###);
    assert_output(&out, r###"float_nan_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/fractions/unsupported_type_raises.py`.
#[test]
fn test_gen_errors_std_libs_fractions_unsupported_type_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fractions"
# dimension = "errors"
# case = "unsupported_type_raises"
# subject = "fractions.Fraction"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fractions.py"
# status = "filled"
# ///
"""fractions.Fraction: unsupported_type_raises (errors)."""
import fractions

_raised = False
try:
    fractions.Fraction([1, 2])
except TypeError:
    _raised = True
assert _raised, "unsupported_type_raises: expected TypeError"
print("unsupported_type_raises OK")
"###);
    assert_output(&out, r###"unsupported_type_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/fractions/zero_denominator_raises.py`.
#[test]
fn test_gen_errors_std_libs_fractions_zero_denominator_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fractions"
# dimension = "errors"
# case = "zero_denominator_raises"
# subject = "fractions.Fraction"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fractions.py"
# status = "filled"
# ///
"""fractions.Fraction: zero_denominator_raises (errors)."""
import fractions

_raised = False
try:
    fractions.Fraction(1, 0)
except ZeroDivisionError:
    _raised = True
assert _raised, "zero_denominator_raises: expected ZeroDivisionError"
print("zero_denominator_raises OK")
"###);
    assert_output(&out, r###"zero_denominator_raises OK
"###);
}

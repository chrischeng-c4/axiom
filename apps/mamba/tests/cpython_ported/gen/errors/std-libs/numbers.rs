use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/numbers/incomplete_integral_subclass_raises.py`.
#[test]
fn test_gen_errors_std_libs_numbers_incomplete_integral_subclass_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "numbers"
# dimension = "errors"
# case = "incomplete_integral_subclass_raises"
# subject = "numbers.Integral"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_abc.py"
# status = "filled"
# ///
"""numbers.Integral: incomplete_integral_subclass_raises (errors)."""
import numbers

_raised = False
try:
    type('WrongInt', (numbers.Integral,), {})()
except TypeError:
    _raised = True
assert _raised, "incomplete_integral_subclass_raises: expected TypeError"
print("incomplete_integral_subclass_raises OK")
"###);
    assert_output(&out, r###"incomplete_integral_subclass_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/numbers/instantiate_complex_raises.py`.
#[test]
fn test_gen_errors_std_libs_numbers_instantiate_complex_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "numbers"
# dimension = "errors"
# case = "instantiate_complex_raises"
# subject = "numbers.Complex"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_abc.py"
# status = "filled"
# ///
"""numbers.Complex: instantiate_complex_raises (errors)."""
import numbers

_raised = False
try:
    numbers.Complex()
except TypeError:
    _raised = True
assert _raised, "instantiate_complex_raises: expected TypeError"
print("instantiate_complex_raises OK")
"###);
    assert_output(&out, r###"instantiate_complex_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/numbers/instantiate_integral_raises.py`.
#[test]
fn test_gen_errors_std_libs_numbers_instantiate_integral_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "numbers"
# dimension = "errors"
# case = "instantiate_integral_raises"
# subject = "numbers.Integral"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_abc.py"
# status = "filled"
# ///
"""numbers.Integral: instantiate_integral_raises (errors)."""
import numbers

_raised = False
try:
    numbers.Integral()
except TypeError:
    _raised = True
assert _raised, "instantiate_integral_raises: expected TypeError"
print("instantiate_integral_raises OK")
"###);
    assert_output(&out, r###"instantiate_integral_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/numbers/instantiate_rational_raises.py`.
#[test]
fn test_gen_errors_std_libs_numbers_instantiate_rational_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "numbers"
# dimension = "errors"
# case = "instantiate_rational_raises"
# subject = "numbers.Rational"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_abc.py"
# status = "filled"
# ///
"""numbers.Rational: instantiate_rational_raises (errors)."""
import numbers

_raised = False
try:
    numbers.Rational()
except TypeError:
    _raised = True
assert _raised, "instantiate_rational_raises: expected TypeError"
print("instantiate_rational_raises OK")
"###);
    assert_output(&out, r###"instantiate_rational_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/numbers/instantiate_real_raises.py`.
#[test]
fn test_gen_errors_std_libs_numbers_instantiate_real_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "numbers"
# dimension = "errors"
# case = "instantiate_real_raises"
# subject = "numbers.Real"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_abc.py"
# status = "filled"
# ///
"""numbers.Real: instantiate_real_raises (errors)."""
import numbers

_raised = False
try:
    numbers.Real()
except TypeError:
    _raised = True
assert _raised, "instantiate_real_raises: expected TypeError"
print("instantiate_real_raises OK")
"###);
    assert_output(&out, r###"instantiate_real_raises OK
"###);
}

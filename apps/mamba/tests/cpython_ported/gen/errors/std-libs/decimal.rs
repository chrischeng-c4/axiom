use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/decimal/construct_bad_tuple_digit_raises_valueerror.py`.
#[test]
fn test_gen_errors_std_libs_decimal_construct_bad_tuple_digit_raises_valueerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "errors"
# case = "construct_bad_tuple_digit_raises_valueerror"
# subject = "decimal.Decimal"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.Decimal: construct_bad_tuple_digit_raises_valueerror (errors)."""
import decimal

_raised = False
try:
    decimal.Decimal((1, (4, 10, 4), 2))
except ValueError:
    _raised = True
assert _raised, "construct_bad_tuple_digit_raises_valueerror: expected ValueError"
print("construct_bad_tuple_digit_raises_valueerror OK")
"###);
    assert_output(&out, r###"construct_bad_tuple_digit_raises_valueerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/decimal/construct_from_none_raises_typeerror.py`.
#[test]
fn test_gen_errors_std_libs_decimal_construct_from_none_raises_typeerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "errors"
# case = "construct_from_none_raises_typeerror"
# subject = "decimal.Decimal"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.Decimal: construct_from_none_raises_typeerror (errors)."""
import decimal

_raised = False
try:
    decimal.Decimal(None)
except TypeError:
    _raised = True
assert _raised, "construct_from_none_raises_typeerror: expected TypeError"
print("construct_from_none_raises_typeerror OK")
"###);
    assert_output(&out, r###"construct_from_none_raises_typeerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/decimal/context_method_str_operand_raises_typeerror.py`.
#[test]
fn test_gen_errors_std_libs_decimal_context_method_str_operand_raises_typeerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "errors"
# case = "context_method_str_operand_raises_typeerror"
# subject = "decimal.Context"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.Context: context_method_str_operand_raises_typeerror (errors)."""
import decimal

_raised = False
try:
    decimal.Context().to_eng_string('10')
except TypeError:
    _raised = True
assert _raised, "context_method_str_operand_raises_typeerror: expected TypeError"
print("context_method_str_operand_raises_typeerror OK")
"###);
    assert_output(&out, r###"context_method_str_operand_raises_typeerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/decimal/conversion_syntax_raises.py`.
#[test]
fn test_gen_errors_std_libs_decimal_conversion_syntax_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "errors"
# case = "conversion_syntax_raises"
# subject = "decimal.Decimal"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.Decimal: conversion_syntax_raises (errors)."""
import decimal

_raised = False
try:
    decimal.Decimal('not_a_number')
except decimal.InvalidOperation:
    _raised = True
assert _raised, "conversion_syntax_raises: expected decimal.InvalidOperation"
print("conversion_syntax_raises OK")
"###);
    assert_output(&out, r###"conversion_syntax_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/decimal/division_by_zero_raises.py`.
#[test]
fn test_gen_errors_std_libs_decimal_division_by_zero_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "errors"
# case = "division_by_zero_raises"
# subject = "decimal.Decimal"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.Decimal: division_by_zero_raises (errors)."""
import decimal

_raised = False
try:
    decimal.Decimal('1') / decimal.Decimal('0')
except decimal.DivisionByZero:
    _raised = True
assert _raised, "division_by_zero_raises: expected decimal.DivisionByZero"
print("division_by_zero_raises OK")
"###);
    assert_output(&out, r###"division_by_zero_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/decimal/exception_hierarchy.py`.
#[test]
fn test_gen_errors_std_libs_decimal_exception_hierarchy() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "errors"
# case = "exception_hierarchy"
# subject = "decimal.DecimalException"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.DecimalException: every decimal signal derives from DecimalException<:ArithmeticError, and the documented aliases hold: DivisionByZero<:ZeroDivisionError, FloatOperation<:TypeError, Overflow<:Rounded,Inexact, Underflow<:Subnormal, ConversionSyntax/DivisionImpossible/InvalidContext<:InvalidOperation, DivisionUndefined<:ZeroDivisionError"""
from decimal import (
    DecimalException, InvalidOperation, DivisionByZero, Overflow, Underflow,
    Subnormal, Inexact, Rounded, Clamped, FloatOperation, ConversionSyntax,
    DivisionImpossible, DivisionUndefined, InvalidContext,
)

# Every signal derives from DecimalException<:ArithmeticError; some also alias
# Python builtins.
assert issubclass(DecimalException, ArithmeticError), "DecimalException <: ArithmeticError"
assert issubclass(DivisionByZero, ZeroDivisionError), "DivisionByZero <: ZeroDivisionError"
assert issubclass(FloatOperation, TypeError), "FloatOperation <: TypeError"
assert issubclass(Overflow, Rounded) and issubclass(Overflow, Inexact), "Overflow <: Rounded,Inexact"
assert issubclass(Underflow, Subnormal), "Underflow <: Subnormal"
assert issubclass(ConversionSyntax, InvalidOperation), "ConversionSyntax <: InvalidOperation"
assert issubclass(DivisionImpossible, InvalidOperation), "DivisionImpossible <: InvalidOperation"
assert issubclass(DivisionUndefined, ZeroDivisionError), "DivisionUndefined <: ZeroDivisionError"
assert issubclass(InvalidContext, InvalidOperation), "InvalidContext <: InvalidOperation"
for _sig in (Subnormal, Inexact, Rounded, Clamped):
    assert issubclass(_sig, DecimalException), f"{_sig.__name__} <: DecimalException"

print("exception_hierarchy OK")
"###);
    assert_output(&out, r###"exception_hierarchy OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/decimal/int_of_infinity_raises_overflowerror.py`.
#[test]
fn test_gen_errors_std_libs_decimal_int_of_infinity_raises_overflowerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "errors"
# case = "int_of_infinity_raises_overflowerror"
# subject = "decimal.Decimal"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.Decimal: int_of_infinity_raises_overflowerror (errors)."""
import decimal

_raised = False
try:
    int(decimal.Decimal('Infinity'))
except OverflowError:
    _raised = True
assert _raised, "int_of_infinity_raises_overflowerror: expected OverflowError"
print("int_of_infinity_raises_overflowerror OK")
"###);
    assert_output(&out, r###"int_of_infinity_raises_overflowerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/decimal/int_of_nan_raises_valueerror.py`.
#[test]
fn test_gen_errors_std_libs_decimal_int_of_nan_raises_valueerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "errors"
# case = "int_of_nan_raises_valueerror"
# subject = "decimal.Decimal"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.Decimal: int_of_nan_raises_valueerror (errors)."""
import decimal

_raised = False
try:
    int(decimal.Decimal('NaN'))
except ValueError:
    _raised = True
assert _raised, "int_of_nan_raises_valueerror: expected ValueError"
print("int_of_nan_raises_valueerror OK")
"###);
    assert_output(&out, r###"int_of_nan_raises_valueerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/decimal/localcontext_bad_capitals_raises_valueerror.py`.
#[test]
fn test_gen_errors_std_libs_decimal_localcontext_bad_capitals_raises_valueerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "errors"
# case = "localcontext_bad_capitals_raises_valueerror"
# subject = "decimal.localcontext"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.localcontext: localcontext_bad_capitals_raises_valueerror (errors)."""
import decimal

_raised = False
try:
    decimal.localcontext(capitals=2)
except ValueError:
    _raised = True
assert _raised, "localcontext_bad_capitals_raises_valueerror: expected ValueError"
print("localcontext_bad_capitals_raises_valueerror OK")
"###);
    assert_output(&out, r###"localcontext_bad_capitals_raises_valueerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/decimal/nan_equality_is_false.py`.
#[test]
fn test_gen_errors_std_libs_decimal_nan_equality_is_false() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "errors"
# case = "nan_equality_is_false"
# subject = "decimal.Decimal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.Decimal: comparing Decimal('NaN') == Decimal('NaN') returns False (not a raise, not NaN), and != is always True"""
from decimal import Decimal

# Comparing Decimal NaN with == returns False, not NaN and not a raise; != is
# always True.
assert (Decimal("NaN") == Decimal("NaN")) is False, "NaN == NaN is False"
assert Decimal("NaN") != Decimal("NaN"), "NaN != NaN is True"

print("nan_equality_is_false OK")
"###);
    assert_output(&out, r###"nan_equality_is_false OK
"###);
}

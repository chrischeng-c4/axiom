# /// script
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

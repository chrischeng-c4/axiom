# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "errors"
# case = "nan_ordering_raises_invalidoperation"
# subject = "decimal.Decimal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.Decimal: ordering a Decimal NaN via the comparison dunder raises InvalidOperation, while the reflected comparison from a non-Decimal partner returns NotImplemented"""
from decimal import Decimal, InvalidOperation
from fractions import Fraction

# Ordering a Decimal NaN via the comparison dunder raises InvalidOperation.
_raised = False
try:
    Decimal("nan").__gt__(Fraction(-9, 123))
except InvalidOperation:
    _raised = True
assert _raised, "NaN ordering should raise InvalidOperation"
# The reflected comparison from a non-Decimal partner is NotImplemented.
assert Fraction(-9, 123).__lt__(Decimal("nan")) is NotImplemented, "Fraction.__lt__(NaN)"

print("nan_ordering_raises_invalidoperation OK")

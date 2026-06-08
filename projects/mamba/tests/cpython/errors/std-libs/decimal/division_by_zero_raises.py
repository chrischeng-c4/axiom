# /// script
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

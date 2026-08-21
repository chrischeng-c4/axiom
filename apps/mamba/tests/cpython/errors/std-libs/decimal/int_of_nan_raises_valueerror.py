# /// script
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

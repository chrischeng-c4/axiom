# /// script
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

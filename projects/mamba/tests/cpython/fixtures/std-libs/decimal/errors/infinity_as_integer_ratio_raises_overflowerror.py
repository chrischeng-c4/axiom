# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "errors"
# case = "infinity_as_integer_ratio_raises_overflowerror"
# subject = "decimal.Decimal"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.Decimal: infinity_as_integer_ratio_raises_overflowerror (errors)."""
import decimal

_raised = False
try:
    decimal.Decimal('inf').as_integer_ratio()
except OverflowError:
    _raised = True
assert _raised, "infinity_as_integer_ratio_raises_overflowerror: expected OverflowError"
print("infinity_as_integer_ratio_raises_overflowerror OK")

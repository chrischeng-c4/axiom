# /// script
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

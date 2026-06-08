# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "errors"
# case = "create_decimal_non_numeric_raises_valueerror"
# subject = "decimal.Context"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.Context: create_decimal_non_numeric_raises_valueerror (errors)."""
import decimal

_raised = False
try:
    decimal.Context().create_decimal(['%'])
except ValueError:
    _raised = True
assert _raised, "create_decimal_non_numeric_raises_valueerror: expected ValueError"
print("create_decimal_non_numeric_raises_valueerror OK")

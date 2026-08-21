# /// script
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

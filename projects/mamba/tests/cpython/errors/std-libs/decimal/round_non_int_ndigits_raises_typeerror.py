# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "errors"
# case = "round_non_int_ndigits_raises_typeerror"
# subject = "decimal.Decimal"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.Decimal: round_non_int_ndigits_raises_typeerror (errors)."""
import decimal

_raised = False
try:
    decimal.Decimal('1.23').__round__('5')
except TypeError:
    _raised = True
assert _raised, "round_non_int_ndigits_raises_typeerror: expected TypeError"
print("round_non_int_ndigits_raises_typeerror OK")

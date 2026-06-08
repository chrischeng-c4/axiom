# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "errors"
# case = "format_non_str_spec_raises_typeerror"
# subject = "decimal.Decimal"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.Decimal: format_non_str_spec_raises_typeerror (errors)."""
import decimal

_raised = False
try:
    decimal.Decimal(1).__format__(b'-020')
except TypeError:
    _raised = True
assert _raised, "format_non_str_spec_raises_typeerror: expected TypeError"
print("format_non_str_spec_raises_typeerror OK")

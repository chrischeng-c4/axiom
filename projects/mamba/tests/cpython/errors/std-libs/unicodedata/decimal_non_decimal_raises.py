# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "errors"
# case = "decimal_non_decimal_raises"
# subject = "unicodedata.decimal"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unicodedata.py"
# status = "filled"
# ///
"""unicodedata.decimal: decimal_non_decimal_raises (errors)."""
import unicodedata

_raised = False
try:
    unicodedata.decimal("A")
except ValueError:
    _raised = True
assert _raised, "decimal_non_decimal_raises: expected ValueError"
print("decimal_non_decimal_raises OK")

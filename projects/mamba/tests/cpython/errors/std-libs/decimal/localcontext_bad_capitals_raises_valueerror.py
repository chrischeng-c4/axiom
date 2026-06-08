# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "errors"
# case = "localcontext_bad_capitals_raises_valueerror"
# subject = "decimal.localcontext"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.localcontext: localcontext_bad_capitals_raises_valueerror (errors)."""
import decimal

_raised = False
try:
    decimal.localcontext(capitals=2)
except ValueError:
    _raised = True
assert _raised, "localcontext_bad_capitals_raises_valueerror: expected ValueError"
print("localcontext_bad_capitals_raises_valueerror OK")

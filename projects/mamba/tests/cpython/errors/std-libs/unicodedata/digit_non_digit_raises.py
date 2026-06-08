# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "errors"
# case = "digit_non_digit_raises"
# subject = "unicodedata.digit"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unicodedata.py"
# status = "filled"
# ///
"""unicodedata.digit: digit_non_digit_raises (errors)."""
import unicodedata

_raised = False
try:
    unicodedata.digit("A")
except ValueError:
    _raised = True
assert _raised, "digit_non_digit_raises: expected ValueError"
print("digit_non_digit_raises OK")

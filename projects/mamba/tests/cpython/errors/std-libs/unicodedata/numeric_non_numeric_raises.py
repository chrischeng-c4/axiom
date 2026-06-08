# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "errors"
# case = "numeric_non_numeric_raises"
# subject = "unicodedata.numeric"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unicodedata.py"
# status = "filled"
# ///
"""unicodedata.numeric: numeric_non_numeric_raises (errors)."""
import unicodedata

_raised = False
try:
    unicodedata.numeric("A")
except ValueError:
    _raised = True
assert _raised, "numeric_non_numeric_raises: expected ValueError"
print("numeric_non_numeric_raises OK")

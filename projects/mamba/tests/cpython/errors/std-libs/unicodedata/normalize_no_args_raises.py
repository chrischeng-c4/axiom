# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "errors"
# case = "normalize_no_args_raises"
# subject = "unicodedata.normalize"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unicodedata.py"
# status = "filled"
# ///
"""unicodedata.normalize: normalize_no_args_raises (errors)."""
import unicodedata

_raised = False
try:
    unicodedata.normalize()
except TypeError:
    _raised = True
assert _raised, "normalize_no_args_raises: expected TypeError"
print("normalize_no_args_raises OK")

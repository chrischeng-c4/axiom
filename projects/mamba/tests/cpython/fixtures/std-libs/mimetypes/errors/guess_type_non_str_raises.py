# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "errors"
# case = "guess_type_non_str_raises"
# subject = "mimetypes.guess_type"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_mimetypes.py"
# status = "filled"
# ///
"""mimetypes.guess_type: guess_type_non_str_raises (errors)."""
import mimetypes

_raised = False
try:
    mimetypes.guess_type(123)
except TypeError:
    _raised = True
assert _raised, "guess_type_non_str_raises: expected TypeError"
print("guess_type_non_str_raises OK")

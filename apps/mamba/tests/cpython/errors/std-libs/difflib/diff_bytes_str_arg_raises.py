# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "errors"
# case = "diff_bytes_str_arg_raises"
# subject = "difflib.diff_bytes"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_difflib.py"
# status = "filled"
# ///
"""difflib.diff_bytes: diff_bytes_str_arg_raises (errors)."""
import difflib

_raised = False
try:
    list(difflib.diff_bytes(difflib.unified_diff, [b"hello"], ["hello"]))
except TypeError:
    _raised = True
assert _raised, "diff_bytes_str_arg_raises: expected TypeError"
print("diff_bytes_str_arg_raises OK")

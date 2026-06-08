# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "errors"
# case = "unified_diff_mixed_str_bytes_raises"
# subject = "difflib.unified_diff"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_difflib.py"
# status = "filled"
# ///
"""difflib.unified_diff: unified_diff_mixed_str_bytes_raises (errors)."""
import difflib

_raised = False
try:
    list(difflib.unified_diff([b"hello"], ["hello"]))
except TypeError:
    _raised = True
assert _raised, "unified_diff_mixed_str_bytes_raises: expected TypeError"
print("unified_diff_mixed_str_bytes_raises OK")

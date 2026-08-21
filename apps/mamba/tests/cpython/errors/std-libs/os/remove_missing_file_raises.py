# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "errors"
# case = "remove_missing_file_raises"
# subject = "os.remove"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_os.py"
# status = "filled"
# ///
"""os.remove: remove_missing_file_raises (errors)."""
import os

_raised = False
try:
    os.remove('/no/such/file_to_remove_xyzzy')
except FileNotFoundError:
    _raised = True
assert _raised, "remove_missing_file_raises: expected FileNotFoundError"
print("remove_missing_file_raises OK")

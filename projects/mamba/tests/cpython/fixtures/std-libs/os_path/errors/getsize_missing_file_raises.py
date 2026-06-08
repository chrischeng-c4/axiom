# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os_path"
# dimension = "errors"
# case = "getsize_missing_file_raises"
# subject = "os.path.getsize"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_posixpath.py"
# status = "filled"
# ///
"""os.path.getsize: getsize_missing_file_raises (errors)."""
import os.path

_raised = False
try:
    os.path.getsize('/no/such/file_xyzzy')
except FileNotFoundError:
    _raised = True
assert _raised, "getsize_missing_file_raises: expected FileNotFoundError"
print("getsize_missing_file_raises OK")

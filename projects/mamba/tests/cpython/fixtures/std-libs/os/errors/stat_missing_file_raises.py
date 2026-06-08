# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "errors"
# case = "stat_missing_file_raises"
# subject = "os.stat"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_os.py"
# status = "filled"
# ///
"""os.stat: stat_missing_file_raises (errors)."""
import os

_raised = False
try:
    os.stat('/no/such/path/xyzzy')
except FileNotFoundError:
    _raised = True
assert _raised, "stat_missing_file_raises: expected FileNotFoundError"
print("stat_missing_file_raises OK")

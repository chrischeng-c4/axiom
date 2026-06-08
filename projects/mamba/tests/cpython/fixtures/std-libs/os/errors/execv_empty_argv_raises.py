# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "errors"
# case = "execv_empty_argv_raises"
# subject = "os.execv"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_os.py"
# status = "filled"
# ///
"""os.execv: execv_empty_argv_raises (errors)."""
import os

_raised = False
try:
    os.execv('dummy', [])
except ValueError:
    _raised = True
assert _raised, "execv_empty_argv_raises: expected ValueError"
print("execv_empty_argv_raises OK")

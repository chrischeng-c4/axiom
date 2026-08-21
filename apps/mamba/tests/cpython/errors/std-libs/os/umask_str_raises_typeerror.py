# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "errors"
# case = "umask_str_raises_typeerror"
# subject = "os.umask"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_os.py"
# status = "filled"
# ///
"""os.umask: umask_str_raises_typeerror (errors)."""
import os

_raised = False
try:
    os.umask('x')
except TypeError:
    _raised = True
assert _raised, "umask_str_raises_typeerror: expected TypeError"
print("umask_str_raises_typeerror OK")

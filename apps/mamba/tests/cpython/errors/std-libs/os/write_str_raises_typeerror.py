# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "errors"
# case = "write_str_raises_typeerror"
# subject = "os.write"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_os.py"
# status = "filled"
# ///
"""os.write: write_str_raises_typeerror (errors)."""
import os

_raised = False
try:
    os.write(1, 'beans')
except TypeError:
    _raised = True
assert _raised, "write_str_raises_typeerror: expected TypeError"
print("write_str_raises_typeerror OK")

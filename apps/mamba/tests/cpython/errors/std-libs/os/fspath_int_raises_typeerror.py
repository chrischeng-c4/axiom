# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "errors"
# case = "fspath_int_raises_typeerror"
# subject = "os.fspath"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_os.py"
# status = "filled"
# ///
"""os.fspath: fspath_int_raises_typeerror (errors)."""
import os

_raised = False
try:
    os.fspath(42)
except TypeError:
    _raised = True
assert _raised, "fspath_int_raises_typeerror: expected TypeError"
print("fspath_int_raises_typeerror OK")

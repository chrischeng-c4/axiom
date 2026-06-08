# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "errors"
# case = "scandir_float_raises_typeerror"
# subject = "os.scandir"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_os.py"
# status = "filled"
# ///
"""os.scandir: scandir_float_raises_typeerror (errors)."""
import os

_raised = False
try:
    os.scandir(1.234)
except TypeError:
    _raised = True
assert _raised, "scandir_float_raises_typeerror: expected TypeError"
print("scandir_float_raises_typeerror OK")

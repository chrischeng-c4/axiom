# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "errors"
# case = "scandir_empty_string_raises"
# subject = "os.scandir"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_os.py"
# status = "filled"
# ///
"""os.scandir: scandir_empty_string_raises (errors)."""
import os

_raised = False
try:
    list(os.scandir(''))
except FileNotFoundError:
    _raised = True
assert _raised, "scandir_empty_string_raises: expected FileNotFoundError"
print("scandir_empty_string_raises OK")

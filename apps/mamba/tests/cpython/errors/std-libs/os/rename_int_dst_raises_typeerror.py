# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "errors"
# case = "rename_int_dst_raises_typeerror"
# subject = "os.rename"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_os.py"
# status = "filled"
# ///
"""os.rename: rename_int_dst_raises_typeerror (errors)."""
import os

_raised = False
try:
    os.rename('/tmp/some_src', 0)
except TypeError:
    _raised = True
assert _raised, "rename_int_dst_raises_typeerror: expected TypeError"
print("rename_int_dst_raises_typeerror OK")

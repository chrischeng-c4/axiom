# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "errors"
# case = "direntry_not_instantiable"
# subject = "os.DirEntry"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_os.py"
# status = "filled"
# ///
"""os.DirEntry: direntry_not_instantiable (errors)."""
import os

_raised = False
try:
    os.DirEntry()
except TypeError:
    _raised = True
assert _raised, "direntry_not_instantiable: expected TypeError"
print("direntry_not_instantiable OK")

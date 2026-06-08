# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "errors"
# case = "unlink_missing_raises"
# subject = "pathlib.Path.unlink"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
"""pathlib.Path.unlink: unlink_missing_raises (errors)."""
import pathlib

_raised = False
try:
    pathlib.Path('/no/such/file_to_unlink').unlink()
except FileNotFoundError:
    _raised = True
assert _raised, "unlink_missing_raises: expected FileNotFoundError"
print("unlink_missing_raises OK")

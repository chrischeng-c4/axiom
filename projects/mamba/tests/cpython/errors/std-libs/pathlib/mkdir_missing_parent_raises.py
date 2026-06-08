# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "errors"
# case = "mkdir_missing_parent_raises"
# subject = "pathlib.Path.mkdir"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
"""pathlib.Path.mkdir: mkdir_missing_parent_raises (errors)."""
import pathlib

_raised = False
try:
    pathlib.Path('/no/such/parent_xyzzy/child').mkdir()
except FileNotFoundError:
    _raised = True
assert _raised, "mkdir_missing_parent_raises: expected FileNotFoundError"
print("mkdir_missing_parent_raises OK")

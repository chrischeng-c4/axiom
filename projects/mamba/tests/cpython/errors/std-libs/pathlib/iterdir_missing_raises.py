# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "errors"
# case = "iterdir_missing_raises"
# subject = "pathlib.Path.iterdir"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
"""pathlib.Path.iterdir: iterdir_missing_raises (errors)."""
import pathlib

_raised = False
try:
    list(pathlib.Path('/no/such/dir/xyzzy_abc').iterdir())
except FileNotFoundError:
    _raised = True
assert _raised, "iterdir_missing_raises: expected FileNotFoundError"
print("iterdir_missing_raises OK")

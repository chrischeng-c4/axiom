# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "errors"
# case = "bytes_argument_raises"
# subject = "pathlib.PurePath"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
"""pathlib.PurePath: bytes_argument_raises (errors)."""
import pathlib

_raised = False
try:
    pathlib.PurePath(b'a')
except TypeError:
    _raised = True
assert _raised, "bytes_argument_raises: expected TypeError"
print("bytes_argument_raises OK")

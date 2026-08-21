# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "errors"
# case = "read_text_missing_raises"
# subject = "pathlib.Path.read_text"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
"""pathlib.Path.read_text: read_text_missing_raises (errors)."""
import pathlib

_raised = False
try:
    pathlib.Path('/no/such/file/xyzzy_abc').read_text()
except FileNotFoundError:
    _raised = True
assert _raised, "read_text_missing_raises: expected FileNotFoundError"
print("read_text_missing_raises OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "errors"
# case = "relative_as_uri_raises"
# subject = "pathlib.PurePosixPath.as_uri"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
"""pathlib.PurePosixPath.as_uri: relative_as_uri_raises (errors)."""
import pathlib

_raised = False
try:
    pathlib.PurePosixPath('a').as_uri()
except ValueError:
    _raised = True
assert _raised, "relative_as_uri_raises: expected ValueError"
print("relative_as_uri_raises OK")

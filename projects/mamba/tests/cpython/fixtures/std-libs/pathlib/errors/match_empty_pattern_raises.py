# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "errors"
# case = "match_empty_pattern_raises"
# subject = "pathlib.PurePosixPath.match"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
"""pathlib.PurePosixPath.match: match_empty_pattern_raises (errors)."""
import pathlib

_raised = False
try:
    pathlib.PurePosixPath('a').match('')
except ValueError:
    _raised = True
assert _raised, "match_empty_pattern_raises: expected ValueError"
print("match_empty_pattern_raises OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "errors"
# case = "parents_out_of_range_raises"
# subject = "pathlib.PurePath.parents"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
"""pathlib.PurePath.parents: parents_out_of_range_raises (errors)."""
import pathlib

_raised = False
try:
    pathlib.PurePath('a/b/c').parents[3]
except IndexError:
    _raised = True
assert _raised, "parents_out_of_range_raises: expected IndexError"
print("parents_out_of_range_raises OK")

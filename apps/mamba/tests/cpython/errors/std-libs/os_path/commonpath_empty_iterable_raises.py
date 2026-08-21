# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os_path"
# dimension = "errors"
# case = "commonpath_empty_iterable_raises"
# subject = "os.path.commonpath"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_posixpath.py"
# status = "filled"
# ///
"""os.path.commonpath: commonpath_empty_iterable_raises (errors)."""
import os.path

_raised = False
try:
    os.path.commonpath([])
except ValueError:
    _raised = True
assert _raised, "commonpath_empty_iterable_raises: expected ValueError"
print("commonpath_empty_iterable_raises OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os_path"
# dimension = "errors"
# case = "commonpath_mixed_abs_rel_raises"
# subject = "os.path.commonpath"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_posixpath.py"
# status = "filled"
# ///
"""os.path.commonpath: commonpath_mixed_abs_rel_raises (errors)."""
import os.path

_raised = False
try:
    os.path.commonpath(['/abs/path', 'rel/path'])
except ValueError:
    _raised = True
assert _raised, "commonpath_mixed_abs_rel_raises: expected ValueError"
print("commonpath_mixed_abs_rel_raises OK")

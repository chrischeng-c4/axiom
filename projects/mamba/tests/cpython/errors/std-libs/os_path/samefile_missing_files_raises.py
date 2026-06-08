# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os_path"
# dimension = "errors"
# case = "samefile_missing_files_raises"
# subject = "os.path.samefile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_posixpath.py"
# status = "filled"
# ///
"""os.path.samefile: samefile_missing_files_raises (errors)."""
import os.path

_raised = False
try:
    os.path.samefile('/no/such/a', '/no/such/b')
except FileNotFoundError:
    _raised = True
assert _raised, "samefile_missing_files_raises: expected FileNotFoundError"
print("samefile_missing_files_raises OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm"
# dimension = "errors"
# case = "open_missing_read_raises_error"
# subject = "dbm.open"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dbm.py"
# status = "filled"
# ///
"""dbm.open: open_missing_read_raises_error (errors)."""
import dbm

_raised = False
try:
    dbm.open("/no/such/dir/no_such_dbm_path_xyz_qwer", "r")
except dbm.error:
    _raised = True
assert _raised, "open_missing_read_raises_error: expected dbm.error"
print("open_missing_read_raises_error OK")

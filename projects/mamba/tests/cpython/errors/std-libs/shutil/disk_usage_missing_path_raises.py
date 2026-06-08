# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "errors"
# case = "disk_usage_missing_path_raises"
# subject = "shutil.disk_usage"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_shutil.py"
# status = "filled"
# ///
"""shutil.disk_usage: disk_usage_missing_path_raises (errors)."""
import shutil

_raised = False
try:
    shutil.disk_usage("/no/such/path_for_usage")
except FileNotFoundError:
    _raised = True
assert _raised, "disk_usage_missing_path_raises: expected FileNotFoundError"
print("disk_usage_missing_path_raises OK")

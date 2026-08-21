# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "errors"
# case = "rmtree_missing_path_raises"
# subject = "shutil.rmtree"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_shutil.py"
# status = "filled"
# ///
"""shutil.rmtree: rmtree_missing_path_raises (errors)."""
import shutil

_raised = False
try:
    shutil.rmtree("/no/such/path_to_rmtree")
except FileNotFoundError:
    _raised = True
assert _raised, "rmtree_missing_path_raises: expected FileNotFoundError"
print("rmtree_missing_path_raises OK")

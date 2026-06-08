# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "errors"
# case = "update_str_raises"
# subject = "hashlib.sha256"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hashlib.py"
# status = "filled"
# ///
"""hashlib.sha256: update_str_raises (errors)."""
import hashlib

_raised = False
try:
    hashlib.sha256().update('not bytes')
except TypeError:
    _raised = True
assert _raised, "update_str_raises: expected TypeError"
print("update_str_raises OK")

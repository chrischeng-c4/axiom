# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "errors"
# case = "update_int_raises"
# subject = "hashlib.sha256"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hashlib.py"
# status = "filled"
# ///
"""hashlib.sha256: update_int_raises (errors)."""
import hashlib

_raised = False
try:
    hashlib.sha256().update(123)
except TypeError:
    _raised = True
assert _raised, "update_int_raises: expected TypeError"
print("update_int_raises OK")

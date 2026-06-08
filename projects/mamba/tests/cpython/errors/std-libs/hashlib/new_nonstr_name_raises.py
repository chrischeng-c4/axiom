# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "errors"
# case = "new_nonstr_name_raises"
# subject = "hashlib.new"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hashlib.py"
# status = "filled"
# ///
"""hashlib.new: new_nonstr_name_raises (errors)."""
import hashlib

_raised = False
try:
    hashlib.new(1)
except TypeError:
    _raised = True
assert _raised, "new_nonstr_name_raises: expected TypeError"
print("new_nonstr_name_raises OK")

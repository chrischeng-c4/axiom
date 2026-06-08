# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "errors"
# case = "new_unknown_algorithm_raises"
# subject = "hashlib.new"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hashlib.py"
# status = "filled"
# ///
"""hashlib.new: new_unknown_algorithm_raises (errors)."""
import hashlib

_raised = False
try:
    hashlib.new('no_such_algorithm')
except ValueError:
    _raised = True
assert _raised, "new_unknown_algorithm_raises: expected ValueError"
print("new_unknown_algorithm_raises OK")

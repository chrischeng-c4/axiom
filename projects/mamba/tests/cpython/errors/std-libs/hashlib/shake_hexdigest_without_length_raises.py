# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "errors"
# case = "shake_hexdigest_without_length_raises"
# subject = "hashlib.shake_128"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hashlib.py"
# status = "filled"
# ///
"""hashlib.shake_128: shake_hexdigest_without_length_raises (errors)."""
import hashlib

_raised = False
try:
    hashlib.shake_128(b'data').hexdigest()
except TypeError:
    _raised = True
assert _raised, "shake_hexdigest_without_length_raises: expected TypeError"
print("shake_hexdigest_without_length_raises OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "errors"
# case = "pbkdf2_unknown_hash_raises"
# subject = "hashlib.pbkdf2_hmac"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hashlib.py"
# status = "filled"
# ///
"""hashlib.pbkdf2_hmac: pbkdf2_unknown_hash_raises (errors)."""
import hashlib

_raised = False
try:
    hashlib.pbkdf2_hmac('no_such_hash', b'password', b'salt', 1)
except ValueError:
    _raised = True
assert _raised, "pbkdf2_unknown_hash_raises: expected ValueError"
print("pbkdf2_unknown_hash_raises OK")

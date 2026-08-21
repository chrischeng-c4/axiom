# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "behavior"
# case = "sha3_256_incremental_and_copy"
# subject = "hashlib.sha3_256"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hashlib.py"
# status = "filled"
# ///
"""hashlib.sha3_256: sha3_256 incremental update equals one-shot, and copy() is an independent snapshot unaffected by later updates"""
import hashlib

_one = hashlib.sha3_256(b"hello world")
_inc = hashlib.sha3_256()
_inc.update(b"hello")
_inc.update(b" world")
assert _one.digest() == _inc.digest(), "sha3_256 incremental == one-shot"
_h = hashlib.sha3_256(b"base")
_c = _h.copy()
_h.update(b"_more")
assert _c.hexdigest() == hashlib.sha3_256(b"base").hexdigest(), "sha3 copy independent"

print("sha3_256_incremental_and_copy OK")

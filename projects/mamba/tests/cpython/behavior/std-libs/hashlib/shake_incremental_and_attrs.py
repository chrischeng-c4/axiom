# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "behavior"
# case = "shake_incremental_and_attrs"
# subject = "hashlib.shake_128"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hashlib.py"
# status = "filled"
# ///
"""hashlib.shake_128: shake_128 incremental update equals one-shot, .name is 'shake_128', and digest_size is 0 because length is per-call"""
import hashlib

_one = hashlib.shake_128(b"hello world")
_inc = hashlib.shake_128()
_inc.update(b"hello")
_inc.update(b" world")
assert _one.digest(16) == _inc.digest(16), "shake incremental == one-shot"
assert hashlib.shake_128().name == "shake_128", "shake_128 .name"
assert hashlib.shake_128().digest_size == 0, "shake digest_size is 0"

print("shake_incremental_and_attrs OK")

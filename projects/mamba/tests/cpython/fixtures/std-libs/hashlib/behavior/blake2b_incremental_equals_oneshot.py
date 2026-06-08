# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "behavior"
# case = "blake2b_incremental_equals_oneshot"
# subject = "hashlib.blake2b"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hashlib.py"
# status = "filled"
# ///
"""hashlib.blake2b: blake2b incremental update('hello')+update(' world') equals the one-shot digest, and .name reports 'blake2b'/'blake2s'"""
import hashlib

_one = hashlib.blake2b(b"hello world")
_inc = hashlib.blake2b()
_inc.update(b"hello")
_inc.update(b" world")
assert _one.digest() == _inc.digest(), "blake2b incremental == one-shot"
assert hashlib.blake2b().name == "blake2b", "blake2b .name"
assert hashlib.blake2s().name == "blake2s", "blake2s .name"

print("blake2b_incremental_equals_oneshot OK")

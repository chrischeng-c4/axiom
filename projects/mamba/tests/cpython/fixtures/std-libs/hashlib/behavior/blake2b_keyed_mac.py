# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "behavior"
# case = "blake2b_keyed_mac"
# subject = "hashlib.blake2b"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hashlib.py"
# status = "filled"
# ///
"""hashlib.blake2b: a keyed blake2b (MAC use) differs from the unkeyed hash of the same data and is deterministic given the same key"""
import hashlib

_unkeyed = hashlib.blake2b(b"message").hexdigest()
_keyed = hashlib.blake2b(b"message", key=b"secret").hexdigest()
assert _unkeyed != _keyed, "keyed blake2b differs from unkeyed"
assert hashlib.blake2b(b"message", key=b"secret").hexdigest() == _keyed, "keyed blake2b deterministic"

print("blake2b_keyed_mac OK")

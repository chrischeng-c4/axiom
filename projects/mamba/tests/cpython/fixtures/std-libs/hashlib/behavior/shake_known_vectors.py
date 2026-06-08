# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "behavior"
# case = "shake_known_vectors"
# subject = "hashlib.shake_128"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hashlib.py"
# status = "filled"
# ///
"""hashlib.shake_128: variable-length hexdigest: shake_128(b'abc').hexdigest(16) and shake_256(b'abc').hexdigest(32) match their reference values"""
import hashlib

assert hashlib.shake_128(b"abc").hexdigest(16) == \
    "5881092dd818bf5cf8a3ddb793fbcba7", "shake_128('abc', 16)"
assert hashlib.shake_256(b"abc").hexdigest(32) == \
    "483366601360a8771c6863080cc4114d8db44530f8f1e1ee4f94ea37e78b5739", \
    "shake_256('abc', 32)"

print("shake_known_vectors OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "behavior"
# case = "pbkdf2_inputs_change_key"
# subject = "hashlib.pbkdf2_hmac"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hashlib.py"
# status = "filled"
# ///
"""hashlib.pbkdf2_hmac: pbkdf2 is deterministic for identical inputs; changing the iteration count or the salt changes the derived key"""
import hashlib

assert hashlib.pbkdf2_hmac("sha256", b"pw", b"salt", 5) == \
    hashlib.pbkdf2_hmac("sha256", b"pw", b"salt", 5), "pbkdf2 deterministic"
_a = hashlib.pbkdf2_hmac("sha256", b"pw", b"salt", 1)
_b = hashlib.pbkdf2_hmac("sha256", b"pw", b"salt", 2)
assert _a != _b, "iteration count changes derived key"
_s1 = hashlib.pbkdf2_hmac("sha256", b"pw", b"salt1", 5)
_s2 = hashlib.pbkdf2_hmac("sha256", b"pw", b"salt2", 5)
assert _s1 != _s2, "salt changes derived key"

print("pbkdf2_inputs_change_key OK")

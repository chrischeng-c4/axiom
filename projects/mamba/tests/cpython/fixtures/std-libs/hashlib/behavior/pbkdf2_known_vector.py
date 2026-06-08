# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "behavior"
# case = "pbkdf2_known_vector"
# subject = "hashlib.pbkdf2_hmac"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hashlib.py"
# status = "filled"
# ///
"""hashlib.pbkdf2_hmac: one iteration of PBKDF2-HMAC-SHA256 over ('password','salt') matches the known prefix 120fb6cffcf8b32c43e7225256c4f837"""
import hashlib

_dk = hashlib.pbkdf2_hmac("sha256", b"password", b"salt", 1)
assert _dk.hex()[:32] == "120fb6cffcf8b32c43e7225256c4f837", f"pbkdf2 1-iter = {_dk.hex()[:32]!r}"

print("pbkdf2_known_vector OK")

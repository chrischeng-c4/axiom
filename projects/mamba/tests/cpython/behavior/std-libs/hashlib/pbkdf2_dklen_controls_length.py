# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "behavior"
# case = "pbkdf2_dklen_controls_length"
# subject = "hashlib.pbkdf2_hmac"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hashlib.py"
# status = "filled"
# ///
"""hashlib.pbkdf2_hmac: default output length equals the hash's digest_size (32 for sha256); dklen= overrides it exactly (16 over sha256, 40 over sha1)"""
import hashlib

assert len(hashlib.pbkdf2_hmac("sha256", b"pw", b"salt", 1)) == 32, "default dklen == 32 for sha256"
assert len(hashlib.pbkdf2_hmac("sha256", b"pw", b"salt", 10, dklen=16)) == 16, "dklen=16"
assert len(hashlib.pbkdf2_hmac("sha1", b"pw", b"salt", 10, dklen=40)) == 40, "dklen=40 over sha1"

print("pbkdf2_dklen_controls_length OK")

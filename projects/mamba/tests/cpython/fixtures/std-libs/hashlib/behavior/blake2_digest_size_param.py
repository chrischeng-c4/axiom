# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "behavior"
# case = "blake2_digest_size_param"
# subject = "hashlib.blake2b"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hashlib.py"
# status = "filled"
# ///
"""hashlib.blake2b: the digest_size= parameter shrinks output exactly: blake2b(digest_size=16) is 16 bytes, blake2s(digest_size=8) is 8 bytes"""
import hashlib

assert len(hashlib.blake2b(b"x", digest_size=16).digest()) == 16, "blake2b digest_size=16"
assert len(hashlib.blake2s(b"x", digest_size=8).digest()) == 8, "blake2s digest_size=8"

print("blake2_digest_size_param OK")

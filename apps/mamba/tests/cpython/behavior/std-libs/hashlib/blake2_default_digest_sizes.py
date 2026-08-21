# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "behavior"
# case = "blake2_default_digest_sizes"
# subject = "hashlib.blake2b"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hashlib.py"
# status = "filled"
# ///
"""hashlib.blake2b: blake2b default digest_size is 64, blake2s default digest_size is 32"""
import hashlib

assert hashlib.blake2b().digest_size == 64, "blake2b default digest_size"
assert hashlib.blake2s().digest_size == 32, "blake2s default digest_size"

print("blake2_default_digest_sizes OK")

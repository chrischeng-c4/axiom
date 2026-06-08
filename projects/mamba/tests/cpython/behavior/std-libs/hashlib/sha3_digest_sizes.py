# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "behavior"
# case = "sha3_digest_sizes"
# subject = "hashlib.sha3_256"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hashlib.py"
# status = "filled"
# ///
"""hashlib.sha3_256: sha3 digest_size matches the trailing bit count / 8: sha3_224=28, sha3_256=32, sha3_384=48, sha3_512=64"""
import hashlib

assert hashlib.sha3_224().digest_size == 28, "sha3_224 digest_size"
assert hashlib.sha3_256().digest_size == 32, "sha3_256 digest_size"
assert hashlib.sha3_384().digest_size == 48, "sha3_384 digest_size"
assert hashlib.sha3_512().digest_size == 64, "sha3_512 digest_size"

print("sha3_digest_sizes OK")

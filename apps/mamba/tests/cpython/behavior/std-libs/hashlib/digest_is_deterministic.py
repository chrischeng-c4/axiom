# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "behavior"
# case = "digest_is_deterministic"
# subject = "hashlib.sha256"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""hashlib.sha256: the same input always produces the same digest: sha256(b'test') equals itself across two constructions"""
import hashlib

assert hashlib.sha256(b"test").digest() == hashlib.sha256(b"test").digest(), "deterministic"

print("digest_is_deterministic OK")

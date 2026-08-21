# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "behavior"
# case = "different_inputs_differ"
# subject = "hashlib.sha256"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""hashlib.sha256: different inputs produce different digests: sha256(b'a') != sha256(b'b')"""
import hashlib

assert hashlib.sha256(b"a").digest() != hashlib.sha256(b"b").digest(), "collision-free"

print("different_inputs_differ OK")

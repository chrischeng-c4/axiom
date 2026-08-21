# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "behavior"
# case = "blake2b_empty_known_vector"
# subject = "hashlib.blake2b"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hashlib.py"
# status = "filled"
# ///
"""hashlib.blake2b: blake2b(b'') hexdigest begins with the known prefix 786a02f742015903c6c6fd852552d272"""
import hashlib

assert hashlib.blake2b(b"").hexdigest()[:32] == \
    "786a02f742015903c6c6fd852552d272", "blake2b('') prefix"

print("blake2b_empty_known_vector OK")

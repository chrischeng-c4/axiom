# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "surface"
# case = "blake2b_is_callable"
# subject = "hashlib.blake2b"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""hashlib.blake2b: blake2b_is_callable (surface)."""
import hashlib

assert callable(hashlib.blake2b)
print("blake2b_is_callable OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "surface"
# case = "sha3_256_is_callable"
# subject = "hashlib.sha3_256"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""hashlib.sha3_256: sha3_256_is_callable (surface)."""
import hashlib

assert callable(hashlib.sha3_256)
print("sha3_256_is_callable OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "surface"
# case = "sha256_is_callable"
# subject = "hashlib.sha256"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""hashlib.sha256: sha256_is_callable (surface)."""
import hashlib

assert callable(hashlib.sha256)
print("sha256_is_callable OK")

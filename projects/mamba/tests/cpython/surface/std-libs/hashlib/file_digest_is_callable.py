# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "surface"
# case = "file_digest_is_callable"
# subject = "hashlib.file_digest"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""hashlib.file_digest: file_digest_is_callable (surface)."""
import hashlib

assert callable(hashlib.file_digest)
print("file_digest_is_callable OK")

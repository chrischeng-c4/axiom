# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "surface"
# case = "blake2s_is_callable"
# subject = "hashlib.blake2s"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""hashlib.blake2s: blake2s_is_callable (surface)."""
import hashlib

assert callable(hashlib.blake2s)
print("blake2s_is_callable OK")

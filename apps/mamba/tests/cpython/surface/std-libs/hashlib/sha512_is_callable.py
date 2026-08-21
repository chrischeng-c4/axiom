# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "surface"
# case = "sha512_is_callable"
# subject = "hashlib.sha512"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""hashlib.sha512: sha512_is_callable (surface)."""
import hashlib

assert callable(hashlib.sha512)
print("sha512_is_callable OK")

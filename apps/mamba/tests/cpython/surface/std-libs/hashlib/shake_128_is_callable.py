# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "surface"
# case = "shake_128_is_callable"
# subject = "hashlib.shake_128"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""hashlib.shake_128: shake_128_is_callable (surface)."""
import hashlib

assert callable(hashlib.shake_128)
print("shake_128_is_callable OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "surface"
# case = "md5_is_callable"
# subject = "hashlib.md5"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""hashlib.md5: md5_is_callable (surface)."""
import hashlib

assert callable(hashlib.md5)
print("md5_is_callable OK")

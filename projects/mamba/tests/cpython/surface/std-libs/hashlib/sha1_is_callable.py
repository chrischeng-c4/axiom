# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "surface"
# case = "sha1_is_callable"
# subject = "hashlib.sha1"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""hashlib.sha1: sha1_is_callable (surface)."""
import hashlib

assert callable(hashlib.sha1)
print("sha1_is_callable OK")

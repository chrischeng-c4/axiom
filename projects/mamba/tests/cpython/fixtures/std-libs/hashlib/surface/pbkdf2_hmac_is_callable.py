# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "surface"
# case = "pbkdf2_hmac_is_callable"
# subject = "hashlib.pbkdf2_hmac"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""hashlib.pbkdf2_hmac: pbkdf2_hmac_is_callable (surface)."""
import hashlib

assert callable(hashlib.pbkdf2_hmac)
print("pbkdf2_hmac_is_callable OK")

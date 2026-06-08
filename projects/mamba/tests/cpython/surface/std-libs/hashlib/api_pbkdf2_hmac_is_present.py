# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "surface"
# case = "api_pbkdf2_hmac_is_present"
# subject = "hashlib.pbkdf2_hmac"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""hashlib.pbkdf2_hmac: api_pbkdf2_hmac_is_present (surface)."""
import hashlib

assert hasattr(hashlib, "pbkdf2_hmac")
print("api_pbkdf2_hmac_is_present OK")

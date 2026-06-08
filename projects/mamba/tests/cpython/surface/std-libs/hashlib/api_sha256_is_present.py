# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "surface"
# case = "api_sha256_is_present"
# subject = "hashlib.sha256"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""hashlib.sha256: api_sha256_is_present (surface)."""
import hashlib

assert hasattr(hashlib, "sha256")
print("api_sha256_is_present OK")

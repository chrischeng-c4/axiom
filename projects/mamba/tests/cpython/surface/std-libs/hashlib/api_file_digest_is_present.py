# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "surface"
# case = "api_file_digest_is_present"
# subject = "hashlib.file_digest"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""hashlib.file_digest: api_file_digest_is_present (surface)."""
import hashlib

assert hasattr(hashlib, "file_digest")
print("api_file_digest_is_present OK")

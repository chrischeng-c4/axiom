# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "surface"
# case = "api_sha1_is_present"
# subject = "hashlib.sha1"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""hashlib.sha1: api_sha1_is_present (surface)."""
import hashlib

assert hasattr(hashlib, "sha1")
print("api_sha1_is_present OK")

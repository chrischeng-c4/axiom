# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "surface"
# case = "api_md5_is_present"
# subject = "hashlib.md5"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""hashlib.md5: api_md5_is_present (surface)."""
import hashlib

assert hasattr(hashlib, "md5")
print("api_md5_is_present OK")

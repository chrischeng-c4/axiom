# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "surface"
# case = "api_shake_256_is_present"
# subject = "hashlib.shake_256"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""hashlib.shake_256: api_shake_256_is_present (surface)."""
import hashlib

assert hasattr(hashlib, "shake_256")
print("api_shake_256_is_present OK")

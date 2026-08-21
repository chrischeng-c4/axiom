# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "surface"
# case = "algorithms_available_attr"
# subject = "hashlib"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""hashlib: algorithms_available_attr (surface)."""
import hashlib

assert hasattr(hashlib, "algorithms_available")
print("algorithms_available_attr OK")

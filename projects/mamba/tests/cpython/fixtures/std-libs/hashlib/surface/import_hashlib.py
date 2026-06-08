# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "surface"
# case = "import_hashlib"
# subject = "hashlib"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""hashlib: import_hashlib (surface)."""
import hashlib

assert hasattr(hashlib, "sha256")
print("import_hashlib OK")

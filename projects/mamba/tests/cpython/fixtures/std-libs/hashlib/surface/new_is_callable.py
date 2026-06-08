# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "surface"
# case = "new_is_callable"
# subject = "hashlib.new"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""hashlib.new: new_is_callable (surface)."""
import hashlib

assert callable(hashlib.new)
print("new_is_callable OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "surface"
# case = "pack_into_is_callable"
# subject = "struct.pack_into"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""struct.pack_into: pack_into_is_callable (surface)."""
import struct

assert callable(struct.pack_into)
print("pack_into_is_callable OK")

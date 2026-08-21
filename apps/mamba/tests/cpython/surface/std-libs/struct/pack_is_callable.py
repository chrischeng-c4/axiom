# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "surface"
# case = "pack_is_callable"
# subject = "struct.pack"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""struct.pack: pack_is_callable (surface)."""
import struct

assert callable(struct.pack)
print("pack_is_callable OK")

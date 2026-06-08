# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "surface"
# case = "unpack_from_is_callable"
# subject = "struct.unpack_from"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""struct.unpack_from: unpack_from_is_callable (surface)."""
import struct

assert callable(struct.unpack_from)
print("unpack_from_is_callable OK")

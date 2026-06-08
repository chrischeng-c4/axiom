# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "surface"
# case = "unpack_is_callable"
# subject = "struct.unpack"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""struct.unpack: unpack_is_callable (surface)."""
import struct

assert callable(struct.unpack)
print("unpack_is_callable OK")

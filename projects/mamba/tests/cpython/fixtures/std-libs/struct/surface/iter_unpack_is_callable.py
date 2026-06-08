# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "surface"
# case = "iter_unpack_is_callable"
# subject = "struct.iter_unpack"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""struct.iter_unpack: iter_unpack_is_callable (surface)."""
import struct

assert callable(struct.iter_unpack)
print("iter_unpack_is_callable OK")

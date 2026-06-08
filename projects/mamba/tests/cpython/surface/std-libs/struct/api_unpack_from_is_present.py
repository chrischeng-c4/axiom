# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "surface"
# case = "api_unpack_from_is_present"
# subject = "struct.unpack_from"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""struct.unpack_from: api_unpack_from_is_present (surface)."""
import struct

assert hasattr(struct, "unpack_from")
print("api_unpack_from_is_present OK")

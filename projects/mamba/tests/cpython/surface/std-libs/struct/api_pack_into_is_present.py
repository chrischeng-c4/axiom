# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "surface"
# case = "api_pack_into_is_present"
# subject = "struct.pack_into"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""struct.pack_into: api_pack_into_is_present (surface)."""
import struct

assert hasattr(struct, "pack_into")
print("api_pack_into_is_present OK")

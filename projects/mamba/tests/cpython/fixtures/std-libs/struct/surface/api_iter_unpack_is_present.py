# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "surface"
# case = "api_iter_unpack_is_present"
# subject = "struct.iter_unpack"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""struct.iter_unpack: api_iter_unpack_is_present (surface)."""
import struct

assert hasattr(struct, "iter_unpack")
print("api_iter_unpack_is_present OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "surface"
# case = "api_struct_is_present"
# subject = "struct.Struct"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""struct.Struct: api_struct_is_present (surface)."""
import struct

assert hasattr(struct, "Struct")
print("api_struct_is_present OK")

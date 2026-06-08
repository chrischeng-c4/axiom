# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "surface"
# case = "api_calcsize_is_present"
# subject = "struct.calcsize"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""struct.calcsize: api_calcsize_is_present (surface)."""
import struct

assert hasattr(struct, "calcsize")
print("api_calcsize_is_present OK")

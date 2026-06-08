# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "surface"
# case = "api_decompressobj_is_present"
# subject = "zlib.decompressobj"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""zlib.decompressobj: api_decompressobj_is_present (surface)."""
import zlib

assert hasattr(zlib, "decompressobj")
print("api_decompressobj_is_present OK")

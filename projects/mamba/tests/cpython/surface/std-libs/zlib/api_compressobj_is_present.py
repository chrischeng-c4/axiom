# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "surface"
# case = "api_compressobj_is_present"
# subject = "zlib.compressobj"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""zlib.compressobj: api_compressobj_is_present (surface)."""
import zlib

assert hasattr(zlib, "compressobj")
print("api_compressobj_is_present OK")

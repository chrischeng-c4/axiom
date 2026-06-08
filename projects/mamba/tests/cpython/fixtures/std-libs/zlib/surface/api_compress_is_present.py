# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "surface"
# case = "api_compress_is_present"
# subject = "zlib.compress"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""zlib.compress: api_compress_is_present (surface)."""
import zlib

assert hasattr(zlib, "compress")
print("api_compress_is_present OK")

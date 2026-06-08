# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "surface"
# case = "api_decompress_is_present"
# subject = "zlib.decompress"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""zlib.decompress: api_decompress_is_present (surface)."""
import zlib

assert hasattr(zlib, "decompress")
print("api_decompress_is_present OK")

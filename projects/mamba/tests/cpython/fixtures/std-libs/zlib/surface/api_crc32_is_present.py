# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "surface"
# case = "api_crc32_is_present"
# subject = "zlib.crc32"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""zlib.crc32: api_crc32_is_present (surface)."""
import zlib

assert hasattr(zlib, "crc32")
print("api_crc32_is_present OK")

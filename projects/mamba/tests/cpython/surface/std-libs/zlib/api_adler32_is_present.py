# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "surface"
# case = "api_adler32_is_present"
# subject = "zlib.adler32"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""zlib.adler32: api_adler32_is_present (surface)."""
import zlib

assert hasattr(zlib, "adler32")
print("api_adler32_is_present OK")

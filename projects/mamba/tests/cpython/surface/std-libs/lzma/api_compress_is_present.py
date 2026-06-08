# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "surface"
# case = "api_compress_is_present"
# subject = "lzma.compress"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""lzma.compress: api_compress_is_present (surface)."""
import lzma

assert hasattr(lzma, "compress")
print("api_compress_is_present OK")

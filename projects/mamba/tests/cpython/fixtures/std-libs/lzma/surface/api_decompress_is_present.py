# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "surface"
# case = "api_decompress_is_present"
# subject = "lzma.decompress"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""lzma.decompress: api_decompress_is_present (surface)."""
import lzma

assert hasattr(lzma, "decompress")
print("api_decompress_is_present OK")

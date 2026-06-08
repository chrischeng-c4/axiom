# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "surface"
# case = "api_lzma_decompressor_is_present"
# subject = "lzma.LZMADecompressor"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""lzma.LZMADecompressor: api_lzma_decompressor_is_present (surface)."""
import lzma

assert hasattr(lzma, "LZMADecompressor")
print("api_lzma_decompressor_is_present OK")

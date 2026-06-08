# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "surface"
# case = "api_lzma_compressor_is_present"
# subject = "lzma.LZMACompressor"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""lzma.LZMACompressor: api_lzma_compressor_is_present (surface)."""
import lzma

assert hasattr(lzma, "LZMACompressor")
print("api_lzma_compressor_is_present OK")

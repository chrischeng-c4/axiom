# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "surface"
# case = "lzmacompressor_is_callable"
# subject = "lzma.LZMACompressor"
# kind = "mechanical"
# xfail = "lzma.LZMACompressor is a sentinel-string stub, not callable (src/runtime/stdlib/lzma_mod.rs:85-86)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""lzma.LZMACompressor: lzmacompressor_is_callable (surface)."""
import lzma

assert callable(lzma.LZMACompressor)
print("lzmacompressor_is_callable OK")

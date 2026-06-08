# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "surface"
# case = "lzmadecompressor_is_callable"
# subject = "lzma.LZMADecompressor"
# kind = "mechanical"
# xfail = "lzma.LZMADecompressor is a sentinel-string stub, not callable (src/runtime/stdlib/lzma_mod.rs:87-88)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""lzma.LZMADecompressor: lzmadecompressor_is_callable (surface)."""
import lzma

assert callable(lzma.LZMADecompressor)
print("lzmadecompressor_is_callable OK")

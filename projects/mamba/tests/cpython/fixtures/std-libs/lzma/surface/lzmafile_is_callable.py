# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "surface"
# case = "lzmafile_is_callable"
# subject = "lzma.LZMAFile"
# kind = "mechanical"
# xfail = "lzma.LZMAFile is a sentinel-string stub, not callable (src/runtime/stdlib/lzma_mod.rs:79-80)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""lzma.LZMAFile: lzmafile_is_callable (surface)."""
import lzma

assert callable(lzma.LZMAFile)
print("lzmafile_is_callable OK")

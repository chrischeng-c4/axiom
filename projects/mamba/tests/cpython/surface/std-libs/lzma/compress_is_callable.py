# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "surface"
# case = "compress_is_callable"
# subject = "lzma.compress"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""lzma.compress: compress_is_callable (surface)."""
import lzma

assert callable(lzma.compress)
print("compress_is_callable OK")

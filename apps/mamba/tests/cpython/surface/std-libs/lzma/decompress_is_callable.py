# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "surface"
# case = "decompress_is_callable"
# subject = "lzma.decompress"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""lzma.decompress: decompress_is_callable (surface)."""
import lzma

assert callable(lzma.decompress)
print("decompress_is_callable OK")

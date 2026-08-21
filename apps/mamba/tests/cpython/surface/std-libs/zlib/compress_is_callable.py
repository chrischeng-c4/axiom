# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "surface"
# case = "compress_is_callable"
# subject = "zlib.compress"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zlib.compress: compress_is_callable (surface)."""
import zlib

assert callable(zlib.compress)
print("compress_is_callable OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "surface"
# case = "decompress_is_callable"
# subject = "zlib.decompress"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zlib.decompress: decompress_is_callable (surface)."""
import zlib

assert callable(zlib.decompress)
print("decompress_is_callable OK")

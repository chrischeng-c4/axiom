# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "surface"
# case = "crc32_is_callable"
# subject = "zlib.crc32"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zlib.crc32: crc32_is_callable (surface)."""
import zlib

assert callable(zlib.crc32)
print("crc32_is_callable OK")

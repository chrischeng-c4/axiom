# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "surface"
# case = "adler32_is_callable"
# subject = "zlib.adler32"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zlib.adler32: adler32_is_callable (surface)."""
import zlib

assert callable(zlib.adler32)
print("adler32_is_callable OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "surface"
# case = "decompressobj_is_callable"
# subject = "zlib.decompressobj"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zlib.decompressobj: decompressobj_is_callable (surface)."""
import zlib

assert callable(zlib.decompressobj)
print("decompressobj_is_callable OK")

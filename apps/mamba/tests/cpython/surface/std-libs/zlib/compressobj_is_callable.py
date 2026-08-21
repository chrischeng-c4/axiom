# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "surface"
# case = "compressobj_is_callable"
# subject = "zlib.compressobj"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zlib.compressobj: compressobj_is_callable (surface)."""
import zlib

assert callable(zlib.compressobj)
print("compressobj_is_callable OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bz2"
# dimension = "surface"
# case = "decompress_is_callable"
# subject = "bz2.decompress"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bz2.decompress: decompress_is_callable (surface)."""
import bz2

assert callable(bz2.decompress)
print("decompress_is_callable OK")

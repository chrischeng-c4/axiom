# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bz2"
# dimension = "surface"
# case = "bz2decompressor_present"
# subject = "bz2"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bz2: bz2decompressor_present (surface)."""
import bz2

assert hasattr(bz2, "BZ2Decompressor")
print("bz2decompressor_present OK")

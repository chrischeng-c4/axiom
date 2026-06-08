# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bz2"
# dimension = "surface"
# case = "bz2compressor_present"
# subject = "bz2"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bz2: bz2compressor_present (surface)."""
import bz2

assert hasattr(bz2, "BZ2Compressor")
print("bz2compressor_present OK")

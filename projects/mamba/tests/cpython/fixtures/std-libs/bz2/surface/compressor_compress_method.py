# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bz2"
# dimension = "surface"
# case = "compressor_compress_method"
# subject = "bz2.BZ2Compressor"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bz2.BZ2Compressor: compressor_compress_method (surface)."""
import bz2

assert hasattr(bz2.BZ2Compressor, "compress")
print("compressor_compress_method OK")

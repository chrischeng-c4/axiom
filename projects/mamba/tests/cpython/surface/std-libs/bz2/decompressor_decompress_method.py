# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bz2"
# dimension = "surface"
# case = "decompressor_decompress_method"
# subject = "bz2.BZ2Decompressor"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bz2.BZ2Decompressor: decompressor_decompress_method (surface)."""
import bz2

assert hasattr(bz2.BZ2Decompressor, "decompress")
print("decompressor_decompress_method OK")

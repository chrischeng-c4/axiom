# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bz2"
# dimension = "surface"
# case = "compressor_flush_method"
# subject = "bz2.BZ2Compressor"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bz2.BZ2Compressor: compressor_flush_method (surface)."""
import bz2

assert hasattr(bz2.BZ2Compressor, "flush")
print("compressor_flush_method OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bz2"
# dimension = "surface"
# case = "decompressor_eof_attr"
# subject = "bz2.BZ2Decompressor"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bz2.BZ2Decompressor: decompressor_eof_attr (surface)."""
import bz2

assert hasattr(bz2.BZ2Decompressor, "eof")
print("decompressor_eof_attr OK")

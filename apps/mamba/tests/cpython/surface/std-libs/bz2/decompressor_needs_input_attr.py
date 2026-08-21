# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bz2"
# dimension = "surface"
# case = "decompressor_needs_input_attr"
# subject = "bz2.BZ2Decompressor"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bz2.BZ2Decompressor: decompressor_needs_input_attr (surface)."""
import bz2

assert hasattr(bz2.BZ2Decompressor, "needs_input")
print("decompressor_needs_input_attr OK")

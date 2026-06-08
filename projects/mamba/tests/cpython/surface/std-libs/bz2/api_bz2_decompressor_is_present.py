# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bz2"
# dimension = "surface"
# case = "api_bz2_decompressor_is_present"
# subject = "bz2.BZ2Decompressor"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""bz2.BZ2Decompressor: api_bz2_decompressor_is_present (surface)."""
import bz2

assert hasattr(bz2, "BZ2Decompressor")
print("api_bz2_decompressor_is_present OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bz2"
# dimension = "surface"
# case = "compress_is_callable"
# subject = "bz2.compress"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bz2.compress: compress_is_callable (surface)."""
import bz2

assert callable(bz2.compress)
print("compress_is_callable OK")

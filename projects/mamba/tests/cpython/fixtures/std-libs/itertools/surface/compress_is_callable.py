# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "surface"
# case = "compress_is_callable"
# subject = "itertools.compress"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""itertools.compress: compress_is_callable (surface)."""
import itertools

assert callable(itertools.compress)
print("compress_is_callable OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gzip"
# dimension = "surface"
# case = "compress_is_callable"
# subject = "gzip.compress"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""gzip.compress: compress_is_callable (surface)."""
import gzip

assert callable(gzip.compress)
print("compress_is_callable OK")

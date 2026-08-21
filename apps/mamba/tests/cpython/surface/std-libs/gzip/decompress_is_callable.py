# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gzip"
# dimension = "surface"
# case = "decompress_is_callable"
# subject = "gzip.decompress"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""gzip.decompress: decompress_is_callable (surface)."""
import gzip

assert callable(gzip.decompress)
print("decompress_is_callable OK")

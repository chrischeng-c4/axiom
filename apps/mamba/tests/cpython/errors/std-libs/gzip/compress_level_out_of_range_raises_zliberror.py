# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gzip"
# dimension = "errors"
# case = "compress_level_out_of_range_raises_zliberror"
# subject = "gzip.compress"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_gzip.py"
# status = "filled"
# ///
"""gzip.compress: compress_level_out_of_range_raises_zliberror (errors)."""
import gzip
import zlib

_raised = False
try:
    gzip.compress(b'x', compresslevel=10)
except zlib.error:
    _raised = True
assert _raised, "compress_level_out_of_range_raises_zliberror: expected zlib.error"
print("compress_level_out_of_range_raises_zliberror OK")

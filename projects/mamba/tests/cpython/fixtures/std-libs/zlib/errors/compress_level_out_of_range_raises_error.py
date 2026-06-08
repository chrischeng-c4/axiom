# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "errors"
# case = "compress_level_out_of_range_raises_error"
# subject = "zlib.compress"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zlib.compress: compress_level_out_of_range_raises_error (errors)."""
import zlib

_raised = False
try:
    zlib.compress(b'x', 10)
except zlib.error:
    _raised = True
assert _raised, "compress_level_out_of_range_raises_error: expected zlib.error"
print("compress_level_out_of_range_raises_error OK")

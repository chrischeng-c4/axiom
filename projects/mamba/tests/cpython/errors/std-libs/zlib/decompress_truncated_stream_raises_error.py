# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "errors"
# case = "decompress_truncated_stream_raises_error"
# subject = "zlib.decompress"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zlib.decompress: decompress_truncated_stream_raises_error (errors)."""
import zlib

_raised = False
try:
    zlib.decompress(zlib.compress(b'hello zlib')[:3])
except zlib.error:
    _raised = True
assert _raised, "decompress_truncated_stream_raises_error: expected zlib.error"
print("decompress_truncated_stream_raises_error OK")

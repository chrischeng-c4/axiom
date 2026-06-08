# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bz2"
# dimension = "errors"
# case = "decompress_bad_stream_raises"
# subject = "bz2.decompress"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bz2.py"
# status = "filled"
# ///
"""bz2.decompress: decompress_bad_stream_raises (errors)."""
import bz2

_raised = False
try:
    bz2.decompress(b"not a bz2 stream")
except OSError:
    _raised = True
assert _raised, "decompress_bad_stream_raises: expected OSError"
print("decompress_bad_stream_raises OK")

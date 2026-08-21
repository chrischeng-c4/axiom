# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gzip"
# dimension = "errors"
# case = "decompress_truncated_trailer_raises_eoferror"
# subject = "gzip.decompress"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_gzip.py"
# status = "filled"
# ///
"""gzip.decompress: decompress_truncated_trailer_raises_eoferror (errors)."""
import gzip

_raised = False
try:
    gzip.decompress(gzip.compress(b'trailer matters')[:-4])
except EOFError:
    _raised = True
assert _raised, "decompress_truncated_trailer_raises_eoferror: expected EOFError"
print("decompress_truncated_trailer_raises_eoferror OK")

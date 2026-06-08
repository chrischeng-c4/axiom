# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "errors"
# case = "bytes_decode_nontext_codec_raises"
# subject = "bytes.decode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
"""bytes.decode: bytes_decode_nontext_codec_raises (errors)."""
import codecs

_raised = False
try:
    b'hello'.decode('quopri_codec')
except LookupError:
    _raised = True
assert _raised, "bytes_decode_nontext_codec_raises: expected LookupError"
print("bytes_decode_nontext_codec_raises OK")

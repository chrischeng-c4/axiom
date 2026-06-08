# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "errors"
# case = "str_encode_nontext_codec_raises"
# subject = "str.encode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
"""str.encode: str_encode_nontext_codec_raises (errors)."""
import codecs

_raised = False
try:
    'msg'.encode('rot_13')
except LookupError:
    _raised = True
assert _raised, "str_encode_nontext_codec_raises: expected LookupError"
print("str_encode_nontext_codec_raises OK")

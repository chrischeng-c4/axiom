# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "errors"
# case = "getencoder_unknown_codec_raises"
# subject = "codecs.getencoder"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
"""codecs.getencoder: getencoder_unknown_codec_raises (errors)."""
import codecs

_raised = False
try:
    codecs.getencoder('__no_such_codec__')
except LookupError:
    _raised = True
assert _raised, "getencoder_unknown_codec_raises: expected LookupError"
print("getencoder_unknown_codec_raises OK")

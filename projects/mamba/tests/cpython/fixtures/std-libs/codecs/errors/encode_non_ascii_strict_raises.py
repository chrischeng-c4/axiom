# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "errors"
# case = "encode_non_ascii_strict_raises"
# subject = "codecs.encode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
"""codecs.encode: encode_non_ascii_strict_raises (errors)."""
import codecs

_raised = False
try:
    codecs.encode('\u2603', 'ascii')
except UnicodeEncodeError:
    _raised = True
assert _raised, "encode_non_ascii_strict_raises: expected UnicodeEncodeError"
print("encode_non_ascii_strict_raises OK")

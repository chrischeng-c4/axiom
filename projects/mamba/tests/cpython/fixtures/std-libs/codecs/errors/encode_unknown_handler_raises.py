# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "errors"
# case = "encode_unknown_handler_raises"
# subject = "codecs.encode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
"""codecs.encode: encode_unknown_handler_raises (errors)."""
import codecs

_raised = False
try:
    codecs.encode('\u2603', 'ascii', 'no_such_handler')
except LookupError:
    _raised = True
assert _raised, "encode_unknown_handler_raises: expected LookupError"
print("encode_unknown_handler_raises OK")

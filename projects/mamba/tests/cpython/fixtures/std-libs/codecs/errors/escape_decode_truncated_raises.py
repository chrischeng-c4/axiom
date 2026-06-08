# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "errors"
# case = "escape_decode_truncated_raises"
# subject = "codecs.escape_decode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
"""codecs.escape_decode: escape_decode_truncated_raises (errors)."""
import codecs

_raised = False
try:
    codecs.escape_decode(b'\\x')
except ValueError:
    _raised = True
assert _raised, "escape_decode_truncated_raises: expected ValueError"
print("escape_decode_truncated_raises OK")

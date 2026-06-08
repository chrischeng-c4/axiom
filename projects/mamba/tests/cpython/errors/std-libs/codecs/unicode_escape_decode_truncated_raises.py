# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "errors"
# case = "unicode_escape_decode_truncated_raises"
# subject = "codecs.unicode_escape_decode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
"""codecs.unicode_escape_decode: unicode_escape_decode_truncated_raises (errors)."""
import codecs

_raised = False
try:
    codecs.unicode_escape_decode(b'\\x0')
except UnicodeDecodeError:
    _raised = True
assert _raised, "unicode_escape_decode_truncated_raises: expected UnicodeDecodeError"
print("unicode_escape_decode_truncated_raises OK")

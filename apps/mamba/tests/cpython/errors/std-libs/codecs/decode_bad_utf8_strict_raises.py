# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "errors"
# case = "decode_bad_utf8_strict_raises"
# subject = "codecs.decode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
"""codecs.decode: decode_bad_utf8_strict_raises (errors)."""
import codecs

_raised = False
try:
    codecs.decode(b'\xff\xfe\xfd', 'utf-8')
except UnicodeDecodeError:
    _raised = True
assert _raised, "decode_bad_utf8_strict_raises: expected UnicodeDecodeError"
print("decode_bad_utf8_strict_raises OK")

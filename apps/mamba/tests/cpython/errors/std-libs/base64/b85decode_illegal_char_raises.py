# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "errors"
# case = "b85decode_illegal_char_raises"
# subject = "base64.b85decode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_base64.py"
# status = "filled"
# ///
"""base64.b85decode: b85decode_illegal_char_raises (errors)."""
import base64

_raised = False
try:
    base64.b85decode(b'0000"')
except ValueError:
    _raised = True
assert _raised, "b85decode_illegal_char_raises: expected ValueError"
print("b85decode_illegal_char_raises OK")

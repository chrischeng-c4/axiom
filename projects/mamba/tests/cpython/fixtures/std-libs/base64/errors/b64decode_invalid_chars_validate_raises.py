# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "errors"
# case = "b64decode_invalid_chars_validate_raises"
# subject = "base64.b64decode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_base64.py"
# status = "filled"
# ///
"""base64.b64decode: b64decode_invalid_chars_validate_raises (errors)."""
import base64
import binascii

_raised = False
try:
    base64.b64decode(b'not_base64!@#$', validate=True)
except binascii.Error:
    _raised = True
assert _raised, "b64decode_invalid_chars_validate_raises: expected binascii.Error"
print("b64decode_invalid_chars_validate_raises OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "errors"
# case = "b32decode_bad_chars_raises"
# subject = "base64.b32decode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""base64.b32decode: b32decode_bad_chars_raises (errors)."""
import base64
import binascii

_raised = False
try:
    base64.b32decode(b'not_b32_chars!', casefold=False)
except binascii.Error:
    _raised = True
assert _raised, "b32decode_bad_chars_raises: expected binascii.Error"
print("b32decode_bad_chars_raises OK")

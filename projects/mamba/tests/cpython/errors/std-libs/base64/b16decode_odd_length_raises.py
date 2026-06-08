# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "errors"
# case = "b16decode_odd_length_raises"
# subject = "base64.b16decode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""base64.b16decode: b16decode_odd_length_raises (errors)."""
import base64
import binascii

_raised = False
try:
    base64.b16decode(b'abc')
except binascii.Error:
    _raised = True
assert _raised, "b16decode_odd_length_raises: expected binascii.Error"
print("b16decode_odd_length_raises OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "errors"
# case = "urlsafe_b64decode_odd_padding_raises"
# subject = "base64.urlsafe_b64decode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""base64.urlsafe_b64decode: urlsafe_b64decode_odd_padding_raises (errors)."""
import base64
import binascii

_raised = False
try:
    base64.urlsafe_b64decode(b'abc')
except binascii.Error:
    _raised = True
assert _raised, "urlsafe_b64decode_odd_padding_raises: expected binascii.Error"
print("urlsafe_b64decode_odd_padding_raises OK")

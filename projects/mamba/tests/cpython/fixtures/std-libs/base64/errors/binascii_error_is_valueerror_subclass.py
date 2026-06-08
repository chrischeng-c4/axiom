# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "errors"
# case = "binascii_error_is_valueerror_subclass"
# subject = "base64.b64decode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_base64.py"
# status = "filled"
# ///
"""base64.b64decode: binascii.Error is a subclass of ValueError so callers can catch either; this is part of the public decode contract"""
import base64
import binascii

assert issubclass(binascii.Error, ValueError), "binascii.Error <: ValueError"
# Therefore a bad-padding decode can be caught as a plain ValueError.
_raised = False
try:
    base64.b64decode(b"abc")
except ValueError:
    _raised = True
assert _raised, "bad padding catchable as ValueError"
print("binascii_error_is_valueerror_subclass OK")

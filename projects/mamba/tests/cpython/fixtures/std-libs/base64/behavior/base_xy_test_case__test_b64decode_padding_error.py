# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "behavior"
# case = "base_xy_test_case__test_b64decode_padding_error"
# subject = "cpython.test_base64.BaseXYTestCase.test_b64decode_padding_error"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_base64.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_base64.py::BaseXYTestCase::test_b64decode_padding_error
"""Auto-ported test: BaseXYTestCase::test_b64decode_padding_error (CPython 3.12 oracle)."""


import unittest
import base64
import binascii
import os
from array import array
from test.support import os_helper
from test.support import script_helper


# --- test body ---

try:
    base64.b64decode(b'abc')
    raise AssertionError('expected binascii.Error')
except binascii.Error:
    pass

try:
    base64.b64decode('abc')
    raise AssertionError('expected binascii.Error')
except binascii.Error:
    pass
print("BaseXYTestCase::test_b64decode_padding_error: ok")

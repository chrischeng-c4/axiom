# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "behavior"
# case = "legacy_base64_test_case__test_decode"
# subject = "cpython.test_base64.LegacyBase64TestCase.test_decode"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_base64.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_base64.py::LegacyBase64TestCase::test_decode
"""Auto-ported test: LegacyBase64TestCase::test_decode (CPython 3.12 oracle)."""


import unittest
import base64
import binascii
import os
from array import array
from test.support import os_helper
from test.support import script_helper


# --- test body ---
from io import BytesIO, StringIO
infp = BytesIO(b'd3d3LnB5dGhvbi5vcmc=')
outfp = BytesIO()
base64.decode(infp, outfp)

assert outfp.getvalue() == b'www.python.org'

try:
    base64.encode(StringIO('YWJj\n'), BytesIO())
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    base64.encode(BytesIO(b'YWJj\n'), StringIO())
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    base64.encode(StringIO('YWJj\n'), StringIO())
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("LegacyBase64TestCase::test_decode: ok")

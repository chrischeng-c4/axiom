# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "behavior"
# case = "base_xy_test_case__test_decode_nonascii_str"
# subject = "cpython.test_base64.BaseXYTestCase.test_decode_nonascii_str"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_base64.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_base64.py::BaseXYTestCase::test_decode_nonascii_str
"""Auto-ported test: BaseXYTestCase::test_decode_nonascii_str (CPython 3.12 oracle)."""


import unittest
import base64
import binascii
import os
from array import array
from test.support import os_helper
from test.support import script_helper


# --- test body ---
decode_funcs = (base64.b64decode, base64.standard_b64decode, base64.urlsafe_b64decode, base64.b32decode, base64.b16decode, base64.b85decode, base64.a85decode)
for f in decode_funcs:

    try:
        f('with non-ascii Ë')
        raise AssertionError('expected ValueError')
    except ValueError:
        pass
print("BaseXYTestCase::test_decode_nonascii_str: ok")

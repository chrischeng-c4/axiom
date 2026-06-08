# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "behavior"
# case = "test_main__test_decode"
# subject = "cpython.test_base64.TestMain.test_decode"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_base64.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_base64.py::TestMain::test_decode
"""Auto-ported test: TestMain::test_decode (CPython 3.12 oracle)."""


import unittest
import base64
import binascii
import os
from array import array
from test.support import os_helper
from test.support import script_helper


# --- test body ---
def get_output(*args):
    return script_helper.assert_python_ok('-m', 'base64', *args).out
with open(os_helper.TESTFN, 'wb') as fp:
    fp.write(b'Yf9iCg==')
output = get_output('-d', os_helper.TESTFN)

assert output.rstrip() == b'a\xffb'
print("TestMain::test_decode: ok")

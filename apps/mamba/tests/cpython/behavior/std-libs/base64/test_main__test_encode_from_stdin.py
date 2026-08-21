# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "behavior"
# case = "test_main__test_encode_from_stdin"
# subject = "cpython.test_base64.TestMain.test_encode_from_stdin"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_base64.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_base64.py::TestMain::test_encode_from_stdin
"""Auto-ported test: TestMain::test_encode_from_stdin (CPython 3.12 oracle)."""


import unittest
import base64
import binascii
import os
from array import array
from test.support import os_helper
from test.support import script_helper


# --- test body ---
with script_helper.spawn_python('-m', 'base64', '-e') as proc:
    out, err = proc.communicate(b'a\xffb\n')

assert out.rstrip() == b'Yf9iCg=='

assert err is None
print("TestMain::test_encode_from_stdin: ok")

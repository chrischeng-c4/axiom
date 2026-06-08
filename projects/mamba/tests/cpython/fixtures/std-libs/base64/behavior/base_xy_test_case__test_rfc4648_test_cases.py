# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "behavior"
# case = "base_xy_test_case__test_rfc4648_test_cases"
# subject = "cpython.test_base64.BaseXYTestCase.test_RFC4648_test_cases"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_base64.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_base64.py::BaseXYTestCase::test_RFC4648_test_cases
"""Auto-ported test: BaseXYTestCase::test_RFC4648_test_cases (CPython 3.12 oracle)."""


import unittest
import base64
import binascii
import os
from array import array
from test.support import os_helper
from test.support import script_helper


# --- test body ---
b64encode = base64.b64encode
b32hexencode = base64.b32hexencode
b32encode = base64.b32encode
b16encode = base64.b16encode

assert b64encode(b'') == b''

assert b64encode(b'f') == b'Zg=='

assert b64encode(b'fo') == b'Zm8='

assert b64encode(b'foo') == b'Zm9v'

assert b64encode(b'foob') == b'Zm9vYg=='

assert b64encode(b'fooba') == b'Zm9vYmE='

assert b64encode(b'foobar') == b'Zm9vYmFy'

assert b32encode(b'') == b''

assert b32encode(b'f') == b'MY======'

assert b32encode(b'fo') == b'MZXQ===='

assert b32encode(b'foo') == b'MZXW6==='

assert b32encode(b'foob') == b'MZXW6YQ='

assert b32encode(b'fooba') == b'MZXW6YTB'

assert b32encode(b'foobar') == b'MZXW6YTBOI======'

assert b32hexencode(b'') == b''

assert b32hexencode(b'f') == b'CO======'

assert b32hexencode(b'fo') == b'CPNG===='

assert b32hexencode(b'foo') == b'CPNMU==='

assert b32hexencode(b'foob') == b'CPNMUOG='

assert b32hexencode(b'fooba') == b'CPNMUOJ1'

assert b32hexencode(b'foobar') == b'CPNMUOJ1E8======'

assert b16encode(b'') == b''

assert b16encode(b'f') == b'66'

assert b16encode(b'fo') == b'666F'

assert b16encode(b'foo') == b'666F6F'

assert b16encode(b'foob') == b'666F6F62'

assert b16encode(b'fooba') == b'666F6F6261'

assert b16encode(b'foobar') == b'666F6F626172'
print("BaseXYTestCase::test_RFC4648_test_cases: ok")
